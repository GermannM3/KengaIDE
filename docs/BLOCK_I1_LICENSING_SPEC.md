# I1-LICENSING — Enterprise Licensing & Entitlement System

> Юридический замок перед продажами. Offline-first, capability-based, audit-integrated.

---

## 0. Позиционирование

**I1-LICENSING ≠ Paywall**

Это:

- юридическая валидность использования
- контроль функций, моделей, режимов
- offline-first (обязательно)
- совместимость с audit / security / installer

---

## 1. Цели

| Цель | Описание |
|------|----------|
| Юридическая валидность | ограничивать использование по договору |
| Управление доступом | функции, модели, режимы |
| Offline-first | работа без интернета |
| Audit integration | все события в I1-AUDIT-LOGGING |
| Enterprise | site / air-gap сценарии |

---

## 2. License Types

| Type | Описание |
|------|----------|
| trial | ограниченный срок / функции |
| personal | один пользователь |
| professional | расширенные функции |
| enterprise | policy-driven, offline, audit |

---

## 3. License Capabilities Model

Лицензия **не бинарная**, а capability-based:

```json
{
  "features": {
    "local_models": true,
    "cloud_models": false,
    "audit_level": "forensic",
    "policy_override": true,
    "plugins": false
  },
  "limits": {
    "projects": 50,
    "agents_per_day": 500
  }
}
```

Совпадает по философии с Security capabilities.

---

## 4. License File Format

### Формат

- `license.json` + `license.sig`
- read-only
- проверяется при старте

### Пример

```json
{
  "license_id": "ENT-2026-00041",
  "type": "enterprise",
  "issued_to": "ACME Corp",
  "valid_from": "2026-01-01",
  "valid_to": "2027-01-01",
  "features": {...},
  "limits": {...},
  "machine_binding": {
    "mode": "optional",
    "hash": "hw+salt"
  }
}
```

---

## 5. Binding Modes

| Mode | Описание |
|------|----------|
| none | portable |
| machine | hash CPU + disk |
| user | OS user |
| site | subnet / license server |

Выбор: во время установки (Installer), фиксируется в audit.

---

## 6. Offline First (жёстко)

- ❌ лицензия **не требует** онлайна
- ❌ никакого periodic phone-home
- ✅ опционально: manual renewal, enterprise license server

---

## 7. Runtime Integration

### Где проверяется

- App startup
- Agent start
- Model selection
- Policy engine
- Installer (mode selection)

### Как

- `LicenseManager`
- `check_capability(feature)`
- результат → Audit

---

## 8. Audit Integration

События:

- `license_loaded`
- `license_validated`
- `license_expired`
- `license_violation`
- `feature_denied`

Все с: license_id, feature, policy result.

---

## 9. UI / UX

### Installer

- экран выбора типа лицензии
- offline import (file / usb)
- enterprise notice

### Runtime

- License status (read-only)
- Expiry warning
- Feature disabled tooltip

---

## 10. Crypto (зарезервировано)

Подписи, key rotation — отдельный подблок при реализации.

Формат `license.sig`: подпись над `license.json` (Ed25519 / RSA-PSS).

---

## 11. Структура в репозитории

```
licensing/
├── license.rs
├── verifier.rs
├── capabilities.rs
├── binding.rs
├── manager.rs
└── README.md
```

---

## 12. Критерий готовности

- лицензия offline
- capability-based
- интегрирована с audit
- enforced в runtime
- installer умеет импорт

---

## 13. Связь блоков

| ← | → |
|---|---|
| I1-AUDIT-LOGGING | I1-UPDATE-SYSTEM — см. docs/BLOCK_I1_UPDATE_SYSTEM_SPEC.md |
| I1-SECURITY | I1-ENTERPRISE-DEPLOY — см. docs/BLOCK_I1_ENTERPRISE_DEPLOY_SPEC.md |

---

## См. также

- `docs/BLOCK_I1_SECURITY_SPEC.md` — capability model
- `docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md` — license events
- `docs/BLOCK_I1_INSTALLER_SPEC.md` — license import
- `.ai/licensing.md` — AI guidance
