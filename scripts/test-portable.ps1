$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $root 'package.json') -Raw | ConvertFrom-Json).version
$name = "FrameFold-$version-windows-x64-lite"
$temporary = Join-Path ([IO.Path]::GetTempPath()) ("FrameFold 中文路径 " + [guid]::NewGuid())
New-Item -ItemType Directory $temporary | Out-Null
$process = $null
try {
    Expand-Archive (Join-Path $root "release/$name.zip") $temporary
    $folder = Join-Path $temporary $name
    & (Join-Path $folder 'binaries/ffmpeg.exe') -hide_banner -f lavfi -i 'testsrc2=size=64x64:rate=1' -frames:v 1 -c:v libx264 -f null -
    if ($LASTEXITCODE -ne 0) { throw 'Shared FFmpeg H.264 smoke test failed' }
    & (Join-Path $folder 'binaries/ffprobe.exe') -version
    if ($LASTEXITCODE -ne 0) { throw 'Shared ffprobe cannot load' }
    $process = Start-Process (Join-Path $folder 'FrameFold.exe') -WorkingDirectory $folder -PassThru
    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Start-Sleep -Seconds 1
        $process.Refresh()
        if ($process.HasExited) { throw "Portable application exited with $($process.ExitCode)" }
        $runtime = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" | Where-Object {
            $_.ParentProcessId -eq $process.Id
        })
        if ($process.MainWindowHandle -ne 0 -and $runtime.Count -gt 0) { $ready = $true; break }
    }
    if (!$ready) { throw 'Portable app did not open a window using the system WebView2 runtime' }
    Write-Host 'Portable ZIP opens a native window with system WebView2 from a Chinese path.'
} finally {
    if ($process -and !$process.HasExited) {
        $null = $process.CloseMainWindow()
        if (!$process.WaitForExit(15000)) { $process.Kill(); $process.WaitForExit() }
    }
    if (Test-Path $temporary) { Remove-Item -Recurse -Force $temporary }
}
