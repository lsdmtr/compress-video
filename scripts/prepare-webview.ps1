$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$version = '153.0.4234.32'
$url = "https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/c3d95bc1-a0a7-4ca6-aaa1-fa0ac3dd1a37/Microsoft.WebView2.FixedVersionRuntime.$version.x64.cab"
$expected = '2cb653a74426f0aa802c2396775c6bc674fd662d5396bd677f47bfa6e12eba9c'
$cache = Join-Path $root '.cache/webview2'
$cab = Join-Path $cache 'runtime.cab'
$destination = Join-Path $root 'src-tauri/WebView2Runtime'
New-Item -ItemType Directory -Force $cache | Out-Null
if (!(Test-Path $cab) -or (Get-FileHash $cab -Algorithm SHA256).Hash -ne $expected) {
    & curl.exe --fail --location --retry 2 --max-time 600 --output "$cab.download" $url
    if ($LASTEXITCODE -ne 0) { throw 'WebView2 download failed' }
    if ((Get-FileHash "$cab.download" -Algorithm SHA256).Hash -ne $expected) { throw 'WebView2 SHA-256 mismatch' }
    Move-Item -Force "$cab.download" $cab
}
$unpack = Join-Path $cache 'unpacked'
if (Test-Path $unpack) { Remove-Item -Recurse -Force $unpack }
New-Item -ItemType Directory $unpack | Out-Null
& "$env:SystemRoot/System32/expand.exe" $cab '-F:*' $unpack | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'WebView2 extraction failed' }
$executables = @(Get-ChildItem $unpack -Recurse -Filter msedgewebview2.exe)
if ($executables.Count -ne 1) { throw 'Unexpected WebView2 archive layout' }
$signature = Get-AuthenticodeSignature $executables[0].FullName
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') {
    throw 'WebView2 must have a valid Microsoft signature'
}
if (Test-Path $destination) { Remove-Item -Recurse -Force $destination }
Copy-Item -Recurse $executables[0].Directory.FullName $destination
@("Microsoft WebView2 Fixed Runtime $version x64", $url, "SHA256: $expected", 'Microsoft license: https://www.microsoft.com/en-us/legal/terms-of-use') | Set-Content (Join-Path $destination 'FRAMEFOLD-SOURCE.txt') -Encoding UTF8
Remove-Item -Recurse -Force $unpack
Write-Host "Verified Microsoft WebView2 runtime: $version"
