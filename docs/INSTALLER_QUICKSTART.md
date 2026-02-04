# Как получить установщик (один файл)

## Сейчас

**Установщик уже можно собрать.** BLOCK 18 реализован.

### Шаги

```bash
# 1. Установить зависимости (если ещё не сделано)
npm install

# 2. Собрать
npm run tauri:build
```

### Где искать результат

**Windows** (сборка на Windows):

- `target/release/bundle/nsis/KengaIDE_0.1.0_x64-setup.exe`

Используй EXE — MSI без EV-сертификата часто блокируется.

**Linux** (сборка на Linux или через CI):

- `target/release/bundle/appimage/KengaIDE_0.1.0_amd64.AppImage`
- `target/release/bundle/deb/kengaide_0.1.0_amd64.deb`

**CI (GitHub Actions):** `.github/workflows/build-release.yml` — при push в `release` или ручном запуске собирает Windows EXE и Linux AppImage, загружает артефакты и создаёт draft-релиз.

### Что получит пользователь

1. Скачивает **один файл** (.exe на Windows, .AppImage на Linux)
2. Запускает → установка
3. Первый запуск → Welcome-экран:
   - «Загрузить модель» (GigaChat / DeepSeek) — нужен интернет, ~4–10 ГБ
   - «Пропустить» — можно добавить OpenAI по API key позже
4. Готово

---

## Ограничения текущей версии

| Что | Статус |
|-----|--------|
| Один файл установщика | ✅ Есть |
| Установка без терминала | ✅ Есть |
| Первый запуск — выбор моделей | ✅ Есть |
| Offline-установка (models в комплекте) | ❌ I1-INSTALLER |
| Enterprise installer (8 экранов, policy) | ❌ I1-INSTALLER |
| Silent install (GPO, apt) | ❌ I1-ENTERPRISE-DEPLOY |

---

## Итого

**Сейчас:** `npm run tauri:build` → один .exe (Windows) или .AppImage (Linux) → отправить человеку → он ставит и пользуется. Модели — при первом запуске (или пропуск + API).

**Позже (I1):** offline bundle, enterprise installer, silent deploy.
