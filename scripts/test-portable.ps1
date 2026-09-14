$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $root 'package.json') -Raw | ConvertFrom-Json).version
$name = "FrameFold-$version-windows-x64-portable"
$temporary = Join-Path ([IO.Path]::GetTempPath()) ("FrameFold 中文路径 " + [guid]::NewGuid())
New-Item -ItemType Directory $temporary | Out-Null
$process = $null
try {
    Expand-Archive (Join-Path $root "release/$name.zip") $temporary
    $folder = Join-Path $temporary $name
    $process = Start-Process (Join-Path $folder 'FrameFold.exe') -WorkingDirectory $folder -PassThru
    $ready = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Start-Sleep -Seconds 1
        $process.Refresh()
        if ($process.HasExited) { throw "Portable application exited with $($process.ExitCode)" }
        $runtime = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" | Where-Object {
            $_.ExecutablePath -and $_.ExecutablePath.StartsWith((Join-Path $folder 'WebView2Runtime'), [StringComparison]::OrdinalIgnoreCase)
        })
        if ($process.MainWindowHandle -ne 0 -and $runtime.Count -gt 0) { $ready = $true; break }
    }
    if (!$ready) { throw 'Portable app did not open a window using its bundled WebView2 runtime' }
    Write-Host 'Portable ZIP opens a native window with bundled WebView2 from a Chinese path.'
} finally {
    if ($process -and !$process.HasExited) {
        $null = $process.CloseMainWindow()
        if (!$process.WaitForExit(15000)) { $process.Kill(); $process.WaitForExit() }
    }
    if (Test-Path $temporary) { Remove-Item -Recurse -Force $temporary }
}
