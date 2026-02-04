# Сборка Linux AppImage

AppImage собирается только на Linux. С Windows используйте GitHub Actions.

## Вариант 1: GitHub Actions (рекомендуется)

### Через веб-интерфейс

1. Запушьте изменения в GitHub.
2. Откройте: **https://github.com/GermannM3/KengaIDE/actions**
3. В списке слева выберите **build-release**.
4. Нажмите **Run workflow** (справа).
5. Дождитесь завершения (примерно 5–10 минут).
6. Откройте последний run → внизу страницы **Artifacts** → скачайте **KengaIDE-Linux-AppImage**.

### Через скрипт (если установлен gh)

```powershell
winget install GitHub.cli
gh auth login
.\scripts\trigger-build-release.ps1
```

## Вариант 2: Локально на Linux

```bash
npm ci
npm run tauri:build
# Результат: src-tauri/target/release/bundle/appimage/KengaIDE_0.1.0_amd64.AppImage
```
