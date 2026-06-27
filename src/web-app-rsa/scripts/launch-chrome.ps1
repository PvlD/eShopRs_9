$paths = @(
    "${env:ProgramFiles}\Google\Chrome\Application\chrome.exe",
    "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe",
    "$env:LOCALAPPDATA\Google\Chrome\Application\chrome.exe"
)

$chrome = $null
foreach ($p in $paths) {
    if (Test-Path $p) { $chrome = $p; break }
}

if (-not $chrome) {
    Write-Error "Chrome not found. Set CHROME_PATH or install Chrome."
    exit 1
}

Write-Host "Launching Chrome with HTTP/3 (QUIC) forced for 127.0.0.1:4433"
& $chrome `
    --enable-quic `
    --origin-to-force-quic-on="127.0.0.1:4433" `
    "https://127.0.0.1:4433/"
