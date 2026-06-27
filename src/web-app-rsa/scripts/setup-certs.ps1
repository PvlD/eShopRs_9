if (!(Get-Command mkcert -ErrorAction SilentlyContinue)) {
    Write-Error "mkcert not found. Install it first: winget install mkcert"
    exit 1
}

$certsDir = Join-Path (Resolve-Path "$PSScriptRoot\..") "config/tls"
New-Item -ItemType Directory -Force -Path $certsDir | Out-Null

$certFile = Join-Path $certsDir "cert.pem"
$keyFile = Join-Path $certsDir "key.pem"

mkcert -ecdsa -cert-file $certFile -key-file $keyFile localhost 127.0.0.1 ::1

Write-Host "Certificates created at:"
Write-Host "  $certFile"
Write-Host "  $keyFile"
Write-Host ""
Write-Host "Chrome will trust these without warning."
