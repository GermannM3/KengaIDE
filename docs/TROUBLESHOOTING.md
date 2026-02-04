# Устранение неполадок

## Ошибка загрузки локальной модели

```
llama_model_load: error loading model: tensor 'blk.X.Y.weight' data is not within the file bounds, model is corrupted or incomplete
```

Файлы модели повреждены или скачаны не полностью. Удалите папку с моделью и скачайте заново. Путь — в настройках или `%APPDATA%\KengaIDE\models` (Windows).

## MSI блокируется Windows

MSI требует EV-сертификат. Используйте EXE: `target/release/bundle/nsis/KengaIDE_*_x64-setup.exe`. См. [SIGNING_WINDOWS.md](SIGNING_WINDOWS.md).

## AppImage на Windows

AppImage собирается только на Linux.
- Локально на Linux: `npm run tauri:build`
- Через CI: GitHub Actions → build-release → Run workflow
