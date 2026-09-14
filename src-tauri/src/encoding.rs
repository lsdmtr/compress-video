use crate::model::*;
use std::path::{Path, PathBuf};
pub struct EncodingPlan {
    pub args: Vec<String>,
    pub encoder: String,
}
pub fn dimensions(
    w: u32,
    h: u32,
    sar: f64,
    rotation: i32,
    res: &str,
) -> Result<(u32, u32), String> {
    if w < 2 || h < 2 || !sar.is_finite() || sar <= 0. {
        return Err("视频尺寸无效".into());
    }
    if rotation.rem_euclid(90) != 0 {
        return Err("暂不支持非 90 度倍数的视频旋转角度".into());
    }
    let (mut w, mut h) = (w as f64 * sar, h as f64);
    if !w.is_finite() || w > 32768. || h > 32768. {
        return Err("视频显示尺寸超出支持范围".into());
    }
    if rotation.rem_euclid(180) == 90 {
        std::mem::swap(&mut w, &mut h)
    }
    let bound = match res {
        "original" => None,
        "720" => Some((1280., 720.)),
        "1080" => Some((1920., 1080.)),
        "1440" => Some((2560., 1440.)),
        _ => return Err("分辨率选项无效".into()),
    };
    let scale = if let Some((bw, bh)) = bound {
        let (bw, bh) = if h > w { (bh, bw) } else { (bw, bh) };
        (bw / w).min(bh / h).min(1.)
    } else {
        1.
    };
    Ok((
        (((w * scale / 2.).floor() as u32) * 2).clamp(2, 32768),
        (((h * scale / 2.).floor() as u32) * 2).clamp(2, 32768),
    ))
}
pub fn build_plan(m: &MediaInfo, s: &Settings, nvidia: bool) -> Result<EncodingPlan, String> {
    if let Some(f) = s.fps {
        if !f.is_finite() || !(1. ..=240.).contains(&f) {
            return Err("帧率必须在 1–240 之间".into());
        }
    }
    if !["h264", "hevc"].contains(&s.codec.as_str())
        || !["auto", "cpu", "nvidia"].contains(&s.device.as_str())
    {
        return Err("编码设置无效".into());
    }
    let quality = match s.quality.as_str() {
        "high" => 0,
        "balanced" => 1,
        "small" => 2,
        _ => return Err("质量选项无效".into()),
    };
    if s.device == "nvidia" && !nvidia {
        return Err("NVIDIA 编码短测未通过，请选择 CPU 或自动".into());
    }
    if s.preserve_hdr
        && m.hdr_metadata.iter().any(|data| {
            let kind = data["side_data_type"].as_str().unwrap_or("");
            kind.contains("DOVI")
                || kind.contains("Dolby Vision")
                || kind.contains("Dynamic HDR")
                || kind.contains("HDR Dynamic")
        })
    {
        return Err(
            "当前仅支持保留静态 HDR10 / HLG，暂不支持 Dolby Vision 或动态 HDR 元数据".into(),
        );
    }
    if s.preserve_hdr && s.codec != "hevc" {
        return Err("保留 HDR 需要 HEVC 编码".into());
    }
    if m.hdr && s.preserve_hdr && s.device == "nvidia" {
        return Err("保留 HDR 当前使用 CPU HEVC 编码，请选择 CPU 或自动".into());
    }
    let gpu = s.device != "cpu" && nvidia && !(m.hdr && s.preserve_hdr);
    let encoder = match (s.codec.as_str(), gpu) {
        ("hevc", true) => "hevc_nvenc",
        ("hevc", false) => "libx265",
        (_, true) => "h264_nvenc",
        _ => "libx264",
    }
    .to_string();
    let (w, h) = dimensions(m.width, m.height, m.sar, m.rotation, &s.resolution)?;
    let mut filters = vec![];
    if m.hdr && !s.preserve_hdr {
        filters.push("zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,sidedata=mode=delete:type=MASTERING_DISPLAY_METADATA,sidedata=mode=delete:type=CONTENT_LIGHT_LEVEL".into());
    }
    filters.push(format!("scale={w}:{h}:flags=lanczos,setsar=1"));
    if let Some(fps) = s.fps {
        filters.push(format!("fps={fps}"));
    }
    let mut args = vec![
        "-map".into(),
        "0:V:0".into(),
        "-map".into(),
        "0:a?".into(),
        "-map_metadata".into(),
        "0".into(),
        "-vf".into(),
        filters.join(","),
        "-c:v".into(),
        encoder.clone(),
    ];
    if s.fps.is_none() {
        args.extend(["-fps_mode", "passthrough"].map(String::from));
    }
    if gpu {
        args.extend(
            [
                "-preset",
                "p6",
                "-rc",
                "vbr",
                "-b:v",
                "0",
                "-cq",
                ["20", "24", "28"][quality],
            ]
            .map(String::from),
        );
    } else {
        args.extend(["-preset", "slow", "-crf", ["18", "23", "28"][quality]].map(String::from));
    }
    args.extend(
        [
            "-pix_fmt",
            if m.hdr && s.preserve_hdr {
                "yuv420p10le"
            } else {
                "yuv420p"
            },
        ]
        .map(String::from),
    );
    if m.hdr && s.preserve_hdr {
        let transfer = m
            .color_transfer
            .as_deref()
            .filter(|v| ["smpte2084", "arib-std-b67"].contains(v))
            .ok_or("HDR 传递函数不受支持")?;
        args.extend(
            [
                "-color_primaries",
                "bt2020",
                "-colorspace",
                "bt2020nc",
                "-color_trc",
                transfer,
            ]
            .map(String::from),
        );
        let mut params = vec![format!(
            "colorprim=bt2020:transfer={transfer}:colormatrix=bt2020nc"
        )];
        for data in &m.hdr_metadata {
            if data["side_data_type"] == "Mastering display metadata" {
                let value = |key: &str, unit: f64| -> Option<u64> {
                    let raw = data[key].as_str()?;
                    let (a, b) = raw.split_once('/').unwrap_or((raw, "1"));
                    Some((a.parse::<f64>().ok()? / b.parse::<f64>().ok()? * unit).round() as u64)
                };
                let coords: Option<Vec<_>> = [
                    "green_x",
                    "green_y",
                    "blue_x",
                    "blue_y",
                    "red_x",
                    "red_y",
                    "white_point_x",
                    "white_point_y",
                ]
                .iter()
                .map(|k| value(k, 50000.))
                .collect();
                if let (Some(c), Some(max), Some(min)) = (
                    coords,
                    value("max_luminance", 10000.),
                    value("min_luminance", 10000.),
                ) {
                    params.push(format!(
                        "master-display=G({},{})B({},{})R({},{})WP({},{})L({max},{min})",
                        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
                    ));
                }
            }
            if data["side_data_type"] == "Content light level metadata" {
                if let (Some(max), Some(avg)) =
                    (data["max_content"].as_u64(), data["max_average"].as_u64())
                {
                    params.push(format!("max-cll={max},{avg}"));
                }
            }
        }
        args.extend(["-x265-params".into(), params.join(":")]);
    } else if m.hdr {
        args.extend(
            [
                "-color_primaries",
                "bt709",
                "-color_trc",
                "bt709",
                "-colorspace",
                "bt709",
            ]
            .map(String::from),
        );
    }
    if s.codec == "hevc" {
        args.extend(["-tag:v", "hvc1"].map(String::from));
    }
    for (i, c) in m.audio_codecs.iter().enumerate() {
        args.extend([
            format!("-c:a:{i}"),
            if c == "aac" { "copy" } else { "aac" }.into(),
        ]);
        if let Some(metadata) = m.audio_metadata.get(i) {
            if let Some(title) = &metadata.title {
                args.extend([
                    format!("-metadata:s:a:{i}"),
                    format!("title={title}"),
                    format!("-metadata:s:a:{i}"),
                    format!("handler_name={title}"),
                ]);
            }
            if let Some(language) = &metadata.language {
                args.extend([format!("-metadata:s:a:{i}"), format!("language={language}")]);
            }
        }
        if c != "aac" {
            args.extend([format!("-b:a:{i}"), "192k".into()]);
        }
    }
    args.extend(["-movflags", "+faststart", "-max_muxing_queue_size", "4096"].map(String::from));
    Ok(EncodingPlan { args, encoder })
}
pub fn unique_output(dir: &Path, name: &str) -> PathBuf {
    let stem = Path::new(name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let mut n = 1;
    loop {
        let suffix = if n == 1 {
            String::new()
        } else {
            format!("_{n}")
        };
        let p = dir.join(format!("{stem}_compressed{suffix}.mp4"));
        if !p.exists() {
            return p;
        }
        n += 1
    }
}
