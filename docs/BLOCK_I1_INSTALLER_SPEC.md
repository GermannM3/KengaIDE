# I1-INSTALLER — KengaIDE Secure Visual Installer

> Audit-level спецификация. Готово для репы, Vercel V0, инвесторов. Не переделывать.

---

## 1. Цель установщика

Создать **полноценный визуальный установщик**, который:

- устанавливает **KengaIDE в изолированной среде**
- **не требует** от пользователя знаний Linux / DevOps
- **не использует** `.bat`, `.sh` как основной UX
- позволяет настроить систему **во время установки**
- соответствует enterprise-ожиданиям (audit, security, UX)

> Установщик — это **часть продукта**, а не утилита.

---

## 2. Ключевые требования

### Обязательные

| Требование | Описание |
|------------|----------|
| GUI | desktop-first, не CLI |
| Пошаговый wizard | 8 экранов (см. Flow) |
| Изоляция | sandbox по умолчанию |
| Проверка окружения | автоматическая, без «установите сами» |
| Конфигурация при установке | режим, AI, audit, пути |
| Enterprise-режим | политики, offline, audit |

### Запрещено

- «Скачайте Docker, потом выполните…»
- README на 40 страниц
- bash / bat как основной сценарий
- ручная правка конфигов после установки

---

## 3. Архитектура

### 3.1 Формат

**Standalone Installer Application**

| Вариант | Приоритет |
|---------|-----------|
| Tauri (Rust + Web UI) | ✅ приоритет |
| Qt | fallback |
| Electron | только если нет выбора |

### 3.2 Изоляция среды

Установщик **создаёт собственную среду выполнения**:

| Вариант | Описание |
|---------|----------|
| Embedded Runtime | по умолчанию — всё в Program Files / opt |
| Internal Container | не внешний Docker, опционально |
| User-space sandbox | portable, без прав админа |

Пользователь **не видит терминал**.

### 3.3 Связь с I1-spec

Пути, manifest, policies — см. `docs/BLOCK_I1_SPEC.md`.  
UX-экраны — см. `docs/BLOCK_I1_UX.md`.

---

## 4. Flow установки (8 экранов)

### Экран 1 — Welcome

- Что такое KengaIDE
- Версия
- Лицензия (Community / Pro / Enterprise)

### Экран 2 — Проверка системы

Автоматически:

- ОС, архитектура
- Права (admin / user)
- Диск, память, AVX2

❌ никаких «установите сами»

### Экран 3 — Режим установки

| Режим | Описание |
|-------|----------|
| Standalone Developer | один пользователь, локальные модели |
| Team / Studio | общие настройки, несколько проектов |
| Enterprise | политики, offline, audit |

### Экран 4 — Изоляция

- Тип sandbox (Embedded / Portable / Container)
- Путь установки (визуальный picker)
- Автообновления (on/off)

### Экран 5 — AI / Security

- Включить AI runtime (on/off)
- Локально / удалённо
- Политика логирования
- Режим audit (on/off)

### Экран 6 — Summary

- Что будет установлено
- Куда
- Какие порты / ресурсы

Кнопка **Install**

### Экран 7 — Installation progress

- Визуальный прогресс
- Лог human-readable (не raw)

### Экран 8 — Finish

- Launch KengaIDE
- Open docs
- Export audit report (PDF / JSON)

---

## 5. Installer + Audit (критично)

После установки генерируется:

```
{install_root}/install/
├── audit.json           # метаданные установки для security review
├── environment.snapshot  # ОС, CPU, RAM, диск
├── checksum.lock        # контрольные суммы
└── install.log          # human-readable лог
```

Это можно:

- показать службе безопасности
- приложить к enterprise-договору

---

## 6. Installer + Security

- Sandbox по умолчанию
- Минимум прав
- AI-модули выключаемы
- No outbound traffic без согласия

---

## 7. Installer + AI (важно)

Installer:

- ❌ **не использует** Cursor для генерации
- ❌ **не генерируется** AI
- ✅ может иметь описанные сценарии в промптах

В коде:

```rust
// AI-NO-TOUCH
// SECURITY-CRITICAL
```

---

## 8. Структура в репозитории

```
installer/
├── core/
│   ├── sandbox.rs
│   ├── system_check.rs
│   └── permissions.rs
├── ui/
│   ├── screens/
│   └── wizard.rs
├── config/
│   └── install.schema.json
├── audit/
│   └── report.rs
└── README.md
```

Связь с KengaIDE: installer — отдельный crate или подпроект, не внутри src-tauri.

---

## 9. Критерий «можно продавать»

KengaIDE можно продать, если:

- установка ≤ 10 минут
- пользователь **не видел терминал**
- после установки всё работает
- есть audit-отчёт
- UX не стыдно показать CTO

> Если установщик выглядит как тулза для админа — **продукта не существует.**

---

## 10. Следующие блоки I1

| Блок | Описание |
|------|----------|
| I1-SECURITY | threat model, zones, capability, policy — см. docs/BLOCK_I1_SECURITY_SPEC.md |
| I1-AUDIT-LOGGING | levels, taxonomy, hash-chain — см. docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md |
| I1-LICENSING | types, capabilities, offline import — см. docs/BLOCK_I1_LICENSING_SPEC.md |
| I1-UPDATE-SYSTEM | side-by-side, rollback — см. docs/BLOCK_I1_UPDATE_SYSTEM_SPEC.md |
| I1-ENTERPRISE-DEPLOY | MSI, deb, silent install — см. docs/BLOCK_I1_ENTERPRISE_DEPLOY_SPEC.md |

---

## См. также

- `docs/BLOCK_I1_SPEC.md` — техническая спецификация (пути, manifest)
- `docs/BLOCK_I1_UX.md` — UX-экраны
- `docs/BLOCK_I1_CURSOR.md` — AI-инструментарий
- `.ai/installer.md` — AI-facing: installer = AI-NO-TOUCH
