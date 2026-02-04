# I1-UPDATE-SYSTEM — Controlled Updates & Rollback

> Не автообновление как в Chrome. Policy-driven, side-by-side, rollback.

---

## 0. Позиционирование

**I1-UPDATE-SYSTEM ≠ Chrome auto-update**

Это:

- контролируемое обновление
- side-by-side версии
- rollback без переустановки
- совместимость с audit / licensing / enterprise

---

## 1. Цели

| Цель | Описание |
|------|----------|
| Безопасность | checksum, подпись, no silent install |
| Откат | rollback за одну операцию |
| Изоляция | модели не трогаются при обновлении runtime |
| Policy | enterprise может отключить авто-проверку |

---

## 2. Версионирование Runtime

```
runtime/
├── 1.3.0/
├── 1.4.0/
└── current -> 1.4.0
```

- Новая версия ставится side-by-side
- `current` — symlink на активную версию
- Модели в `models/` — общие, не привязаны к версии runtime

---

## 3. Update Flow

### Проверка обновлений

- опционально (policy: `allow_update_check`)
- при старте или по запросу
- endpoint: версионированный URL, не произвольный

### Скачивание

- только по явному действию пользователя
- checksum обязателен
- resume (Range) поддерживается

### Установка

- распаковка в `runtime/{version}/`
- переключение symlink `current`
- audit: `update_installed`, `version`, `checksum`

### Rollback

- переключение symlink на предыдущую версию
- audit: `rollback_triggered`, `from`, `to`

---

## 4. Policy Integration

```json
{
  "allow_update_check": true,
  "allow_auto_download": false,
  "allowed_channels": ["stable"],
  "rollback_requires": "user"
}
```

Enterprise: `allow_update_check: false` — только manual / air-gap.

---

## 5. Audit Integration

События:

- `update_check`
- `update_available`
- `update_download_started`
- `update_download_completed`
- `update_installed`
- `rollback_triggered`

Все с: version, checksum, timestamp.

---

## 6. UI / UX

### Runtime

- Status bar: текущая версия
- Меню: «Проверить обновления»
- Диалог: новая версия доступна → «Скачать» / «Позже»
- После скачивания: «Установить» / «Отмена»
- «Откатить на предыдущую версию» (если есть)

### Installer

- Не обновляет. Обновление — через runtime или отдельный updater.

---

## 7. Offline / Air-gap

- Обновление из локального файла (`.msi`, `.AppImage`, `.deb`)
- Checksum в комплекте
- Без сетевого доступа

---

## 8. AI Rules (.ai/update.md)

AI:

- ❌ не меняет update logic
- ❌ не обходит checksum
- ❌ не трогает rollback
- ✅ может объяснять, документировать

---

## 9. Структура в репозитории

```
updater/
├── checker.rs
├── downloader.rs
├── installer.rs
├── rollback.rs
└── README.md
```

Или: модуль внутри `installer/` при малом объёме.

---

## 10. Критерий готовности

- side-by-side работает
- rollback за одну операцию
- checksum обязателен
- policy-driven
- audit-события

---

## 11. Связь блоков

| ← | → |
|---|---|
| I1-LICENSING | I1-ENTERPRISE-DEPLOY — см. docs/BLOCK_I1_ENTERPRISE_DEPLOY_SPEC.md |
| I1-INSTALLER | — |

---

## См. также

- `docs/BLOCK_I1_SPEC.md` — секция 9 (версионирование)
- `docs/BLOCK_I1_INSTALLER_SPEC.md` — installer ≠ updater
- `docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md` — update events
- `.ai/update.md` — AI guidance
