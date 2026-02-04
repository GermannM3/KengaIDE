# Сборка Linux AppImage и Windows EXE через CI

Workflow **build-release** собирает оба установщика параллельно.

## Где взять EXE и AppImage

После запуска workflow (вручную или push в `release`):

1. Откройте **Actions** → последний run **build-release**
2. Внизу страницы — блок **Artifacts**
3. Скачайте:
   - **KengaIDE-Windows-x64** — EXE для Windows
   - **KengaIDE-Linux-AppImage** — AppImage для Linux

Файлы не попадают в репозиторий — они только в артефактах run. Чтобы они появились в **Releases**, запушьте изменения в ветку `release` — тогда создастся draft-релиз с обоими файлами.

## Запуск workflow

### Вручную

1. **https://github.com/GermannM3/KengaIDE/actions**
2. Слева выберите **build-release**
3. **Run workflow** → Run workflow
4. Подождите 5–10 минут

### Через скрипт (если установлен gh)

```powershell
gh auth login
.\scripts\trigger-build-release.ps1
```

## Локально на Linux

```bash
npm ci
npm run tauri:build
# Результат: src-tauri/target/release/bundle/appimage/KengaIDE_0.1.0_amd64.AppImage
```
