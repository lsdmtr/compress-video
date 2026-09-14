"""Prepare/assemble a Windows test ZIP locally on macOS, never invoke cloud CI."""
import hashlib
import json
import pathlib
import shutil
import struct
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent



def digest(file):
    result = hashlib.sha256()
    with file.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def verify_pe(file):
    with file.open('rb') as stream:
        if stream.read(2) != b'MZ':
            raise RuntimeError(f'Not a Windows executable: {file.name}')
        stream.seek(0x3c)
        offset = struct.unpack('<I', stream.read(4))[0]
        stream.seek(offset)
        if stream.read(6) != b'PE\0\0\x64\x86':
            raise RuntimeError(f'Not a Windows x64 executable: {file.name}')


def package():
    version = json.loads((ROOT / 'package.json').read_text())['version']
    name = f'FrameFold-{version}-windows-x64-lite'
    output = ROOT / 'release'
    stage = output / name
    if stage.exists():
        shutil.rmtree(stage)
    (stage / 'binaries').mkdir(parents=True)
    shutil.copy2(ROOT / 'src-tauri/target/x86_64-pc-windows-msvc/release/framefold.exe', stage / 'FrameFold.exe')
    dlls = sorted(file.name for file in (ROOT / 'src-tauri/binaries').glob('*.dll'))
    if not dlls:
        raise RuntimeError('Missing shared FFmpeg libraries; run npm run prepare:windows')
    for file in ['ffmpeg.exe', 'ffprobe.exe', 'FFMPEG-LICENSE.txt', 'FFMPEG-README.txt', 'FFMPEG-SOURCE.txt', *dlls]:
        shutil.copy2(ROOT / 'src-tauri/binaries' / file, stage / 'binaries' / file)
    shutil.copy2(ROOT / 'docs/portable-readme.txt', stage / 'README.txt')
    for executable in [stage / 'FrameFold.exe', stage / 'binaries/ffmpeg.exe',
                       stage / 'binaries/ffprobe.exe', *[stage / 'binaries' / file for file in dlls]]:
        verify_pe(executable)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    (stage / 'BUILD-INFO.txt').write_text(
        f'FrameFold {version}\nGit commit: {revision}\nProfile: Release, Windows x64 MSVC\n'
        'Built locally on macOS using cargo-xwin. Application is unsigned.\n'
        'Requires the Microsoft WebView2 Runtime already installed on the computer.\n'
        'Windows Defender scanning and native Windows startup were NOT performed.\n'
        'For tester evaluation; report any security warning without disabling protection.\n', encoding='utf-8')
    files = sorted(file for file in stage.rglob('*') if file.is_file())
    (stage / 'SHA256SUMS.txt').write_text(''.join(
        f'{digest(file)}  {file.relative_to(stage).as_posix()}\n' for file in files), encoding='utf-8')
    archive = output / (name + '.zip')
    partial = output / (name + '.zip.partial')
    with zipfile.ZipFile(partial, 'w', zipfile.ZIP_DEFLATED, compresslevel=6) as target:
        for file in sorted(stage.rglob('*')):
            if file.is_file():
                target.write(file, file.relative_to(output).as_posix())
    with zipfile.ZipFile(partial) as target:
        failure = target.testzip()
        if failure:
            raise RuntimeError(f'ZIP verification failed: {failure}')
    partial.replace(archive)
    (output / (name + '.zip.sha256')).write_text(f'{digest(archive)}  {archive.name}\n')
    print(f'Local Release ZIP verified: {archive} ({archive.stat().st_size} bytes)')


if __name__ == '__main__':
    {'package': package}[sys.argv[1]]()
