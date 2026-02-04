# KengaIDE - create self-signed cert for development
# Run: PowerShell -ExecutionPolicy Bypass -File scripts/create-dev-cert.ps1
# Cert is valid only on this machine. For distribution use a CA certificate.

$certName = "KengaIDE Development"
$certPath = "$PSScriptRoot\..\kengaide-dev.pfx"
$tauriConf = "$PSScriptRoot\..\src-tauri\tauri.conf.json"

Write-Host "Creating self-signed certificate for KengaIDE..." -ForegroundColor Cyan

# Создаём самоподписанный сертификат для подписи кода
$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject "CN=KengaIDE" `
    -FriendlyName $certName `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(3) `
    -KeyAlgorithm RSA `
    -KeyLength 2048 `
    -HashAlgorithm SHA256

# Export to PFX (password "dev" for development backup)
$password = ConvertTo-SecureString -String "dev" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath $certPath -Password $password -Force

# Добавляем в Trusted Root — тогда Windows не будет показывать "Неизвестный издатель" на этой машине
$rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store("Root", "CurrentUser")
$rootStore.Open("ReadWrite")
$rootStore.Add($cert)
$rootStore.Close()

# Обновляем tauri.conf.json (точечная замена, без потери структуры)
$content = Get-Content $tauriConf -Raw
$content = $content -replace '"certificateThumbprint":\s*null', "`"certificateThumbprint`": `"$($cert.Thumbprint)`""
$content = $content -replace '"timestampUrl":\s*""', '"timestampUrl": "http://timestamp.digicert.com"'
Set-Content $tauriConf -Value $content -Encoding UTF8

Write-Host ""
Write-Host "Certificate created and added to Trusted Root." -ForegroundColor Green
Write-Host "tauri.conf.json updated (certificateThumbprint, timestampUrl)." -ForegroundColor Green
Write-Host ""
Write-Host "Run: npm run tauri:build" -ForegroundColor Cyan
