$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $root 'package.json') -Raw | ConvertFrom-Json).version
$name = "FrameFold-$version-windows-x64-lite"
$release = Join-Path $root 'release'
$stage = Join-Path $release $name
$zip = Join-Path $release "$name.zip"
foreach ($old in @($zip, "$zip.sha256")) {
    if (Test-Path $old) { Remove-Item -Force $old }
}
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item (Join-Path $root 'src-tauri/target/x86_64-pc-windows-msvc/release/framefold.exe') (Join-Path $stage 'FrameFold.exe')
New-Item -ItemType Directory (Join-Path $stage 'binaries') | Out-Null
$dlls = @(Get-ChildItem (Join-Path $root 'src-tauri/binaries') -Filter '*.dll' -File | ForEach-Object { $_.Name })
if ($dlls.Count -eq 0) { throw 'Shared FFmpeg libraries are missing' }
foreach ($file in (@('ffmpeg.exe', 'ffprobe.exe', 'FFMPEG-LICENSE.txt', 'FFMPEG-README.txt', 'FFMPEG-SOURCE.txt') + $dlls)) {
    Copy-Item (Join-Path $root "src-tauri/binaries/$file") (Join-Path $stage "binaries/$file")
}
Copy-Item (Join-Path $root 'docs/portable-readme.txt') (Join-Path $stage 'README.txt')
# Optional signing uses an existing certificate in the build machine's store.
# Never rewrite or strip the publisher signatures of bundled dependencies.
if ($env:FRAMEFOLD_SIGN_THUMBPRINT) {
    & signtool.exe sign /sha1 $env:FRAMEFOLD_SIGN_THUMBPRINT /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 (Join-Path $stage 'FrameFold.exe')
    if ($LASTEXITCODE -ne 0) { throw 'Application signing failed' }
    if ((Get-AuthenticodeSignature (Join-Path $stage 'FrameFold.exe')).Status -ne 'Valid') { throw 'Invalid application signature' }
}
# Fail closed if Defender is unavailable or finds a threat; do not add exclusions.
$defender = Join-Path $env:ProgramFiles 'Windows Defender/MpCmdRun.exe'
if (!(Test-Path $defender)) { throw 'Microsoft Defender is required for package verification' }
& $defender -SignatureUpdate
if ($LASTEXITCODE -ne 0) { throw 'Defender signature update failed' }
& $defender -Scan -ScanType 3 -File $stage -DisableRemediation
if ($LASTEXITCODE -ne 0) { throw 'Defender scan failed or detected a threat; package withheld' }
$hashes = Get-ChildItem $stage -File -Recurse | Sort-Object FullName | ForEach-Object {
    '{0}  {1}' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $_.FullName.Substring($stage.Length + 1).Replace([IO.Path]::DirectorySeparatorChar, [char]'/')
}
$hashes | Set-Content (Join-Path $stage 'SHA256SUMS.txt') -Encoding UTF8
& tar.exe -a -cf $zip -C $release $name
if ($LASTEXITCODE -ne 0) {
    if (Test-Path $zip) { Remove-Item -Force $zip }
    throw 'Portable ZIP creation failed'
}
'{0}  {1}' -f (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant(), "$name.zip" | Set-Content "$zip.sha256" -Encoding UTF8
Write-Host "Portable package verified: $zip"
