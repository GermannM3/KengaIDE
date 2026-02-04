# KengaIDE

IDE с встроенным AI-рантаймом. Фундамент по зафиксированной архитектуре.

## Стек

- **UI**: Tauri 2, React, TypeScript, Monaco Editor
- **Backend**: Rust (tokio, serde)
- **AI**: провайдеры (Local / API), Context Manager, Model Manager

## Структура

```
KengaIDE/
├── crates/           # Rust workspace
│   ├── backend_core  # FS, Project, Git, Command Router
│   ├── context_manager
│   ├── model_manager
│   ├── ai_providers   # LocalProvider, ApiProvider
│   └── ai_runtime
├── src-tauri/        # Tauri app
├── src/              # React frontend
└── ARCHITECTURE.md    # Зафиксированная архитектура
```

## Сборка

```bash
# Зависимости
npm install

# Иконки (для bundle)
# Создай PNG 1024x1024 и выполни:
# npx tauri icon path/to/icon.png

# Разработка
npm run tauri dev

# Сборка (установщик)
npm run tauri:build
```

## Установщик (один файл)

После `npm run tauri:build` (на Windows):

- **Windows:** `target/release/bundle/nsis/KengaIDE_0.1.0_x64-setup.exe`

Рекомендуется использовать **EXE (NSIS)** — MSI часто блокируется Windows без EV-сертификата.

**Linux (AppImage):** сборка только на Linux. Варианты:
- Собрать локально на Linux: `npm run tauri:build` → `.../bundle/appimage/*.AppImage`
- **CI (рекомендуется):** `.\scripts\trigger-build-release.ps1` или GitHub → Actions → build-release → Run workflow → скачать артефакт KengaIDE-Linux-AppImage

Отправь файл — человек ставит и пользуется. При первом запуске: выбор моделей для докачки или «Пропустить».

### Подпись (Windows)

Без подписи Windows показывает «Неизвестный издатель». Для разработки:

```powershell
PowerShell -ExecutionPolicy Bypass -File scripts/create-dev-cert.ps1
npm run tauri:build
```

Для распространения нужен сертификат от CA — см. [docs/SIGNING_WINDOWS.md](docs/SIGNING_WINDOWS.md).

Подробнее: [docs/INSTALLER_QUICKSTART.md](docs/INSTALLER_QUICKSTART.md)

## Текущее состояние

- Backend Core, AI Runtime, Providers — скелет готов
- LocalProvider, ApiProvider — заглушки (модель не загружена, API не подключён)
- UI — Monaco + панель AI, вызов `ai_request`
- Model Manager — заглушка
# KengaIDE
