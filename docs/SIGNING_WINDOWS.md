# Подпись приложения для Windows

Без подписи Windows показывает «Неизвестный издатель» при установке. Есть два варианта.

---

## 1. Разработка (самоподписанный сертификат)

Подходит только для **вашей машины**. Установщик будет доверенным только там, где создан сертификат.

```powershell
PowerShell -ExecutionPolicy Bypass -File scripts/create-dev-cert.ps1
npm run tauri:build
```

Скрипт:
- создаёт самоподписанный сертификат;
- добавляет его в Trusted Root (чтобы Windows не показывала предупреждение);
- обновляет `tauri.conf.json` (certificateThumbprint, timestampUrl).

После этого сборка подписывает EXE (NSIS). MSI в targets не включён — он часто блокируется без EV-сертификата.

---

## 2. Распространение (сертификат от CA)

Для распространения нужен **код-подписывающий сертификат** от доверенного центра (DigiCert, Sectigo и т.п.).

### Шаги

1. Купить сертификат (обычно $100–400/год).
2. Получить `.cer` и приватный ключ.
3. Собрать PFX:
   ```bash
   openssl pkcs12 -export -in cert.cer -inkey private-key.key -out certificate.pfx
   ```
4. Импортировать в Windows:
   ```powershell
   Import-PfxCertificate -FilePath certificate.pfx -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString -String "PASSWORD" -Force -AsPlainText)
   ```
5. В `certmgr.msc` → Personal → Certificates найти сертификат и скопировать **Thumbprint**.
6. В `tauri.conf.json`:
   ```json
   "windows": {
     "certificateThumbprint": "A1B2C3...",
     "digestAlgorithm": "sha256",
     "timestampUrl": "http://timestamp.digicert.com"
   }
   ```
7. Собрать: `npm run tauri:build`

### EV vs OV

- **EV** — сразу доверенный SmartScreen, дороже.
- **OV** — дешевле, репутация нарабатывается со временем; можно отправить файл в [Microsoft](https://www.microsoft.com/en-us/wdsi/filesubmission/) для проверки.

---

## Ссылки

- [Tauri: Windows Code Signing](https://tauri.app/distribute/sign/windows/)
- [Microsoft: Code Signing](https://learn.microsoft.com/en-us/windows-hardware/drivers/dashboard/code-signing-cert-manage)
