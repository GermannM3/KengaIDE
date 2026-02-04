# BLOCK 18 — Installer (Windows + Linux)

## Цель
Пользователь получает готовый установщик, устанавливает приложение, при первом запуске выбирает модели для докачки.

## Компоненты

### 1. Windows
- MSI (.msi) — WiX Toolset, только на Windows
- NSIS (-setup.exe) — можно кросс-компилировать
- WebView2: downloadBootstrapper по умолчанию (меньший размер)
- targets: "all" → оба формата

### 2. Linux
- AppImage — портативный, без установки
- .deb — для Debian/Ubuntu
- Сборка только на Linux (кросс-компиляция не поддерживается)

### 3. Первый запуск
- При отсутствии локальных моделей — Welcome/Setup экран
- Проверка: есть ли хотя бы одна модель
- Выбор: GigaChat (RU), DeepSeek-Coder (coding), или обе
- Докачка выбранных моделей
- После успешной загрузки — переход в основное окно

### 4. Build scripts
- `npm run tauri build` — полная сборка
- `npm run tauri build -- --target msi` — только MSI (Windows)
- `npm run tauri build -- --target nsis` — только NSIS (Windows)
