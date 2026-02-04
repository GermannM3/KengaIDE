# I1-AUDIT-LOGGING — Security-Grade Audit & Compliance

> Расширение E6. Enterprise-уровень: SOC2 / ISO / внутренний аудит.

---

## 0. Позиционирование

**I1-AUDIT-LOGGING = расширение E6**, но:

- жёстче
- формальнее
- юридически пригодно
- installer + runtime + security

---

## 1. Цели

| Цель | Описание |
|------|----------|
| Reproducibility | восстановить ход событий |
| Explainability | почему принято решение |
| Compliance | доказуемость для enterprise |
| Tamper resistance | защита от подмены |
| AI accountability | кто, что, по какой политике |

---

## 2. Audit Levels

| Level | Описание |
|-------|----------|
| off | только crash logs (dev) |
| basic | high-level события |
| full | все действия агента + security |
| forensic | + хэши, тайминги, env snapshot |

Источник: installer, `policies.json`, enterprise override.

---

## 3. Audit Storage Model

### Пути (I1)

| Режим | Audit path |
|-------|------------|
| Installed | system: read-only, audit: append-only |
| Portable | рядом с data dir |

### Форматы

| Файл | Назначение |
|------|------------|
| `session_id.jsonl` | события |
| `session_id.meta.json` | мета |
| `audit.index` | ускорение поиска |
| `audit.chain` | хэш-цепочка |

---

## 4. Event Taxonomy

### Core

- `session_start`
- `session_end`
- `panic`
- `crash`

### AI (E6 + расширение)

- `task_received`
- `task_classified`
- `model_selected`
- `prompt_sent`
- `tool_call`
- `tool_result`
- `patch_applied`
- `ai_refused`

### Security

- `policy_loaded`
- `policy_denied`
- `capability_granted`
- `capability_denied`
- `sandbox_violation`
- `network_blocked`

### Installer

- `preflight_check`
- `model_installed`
- `checksum_verified`
- `rollback_triggered`

### Update (I1-UPDATE-SYSTEM)

- `update_check`
- `update_available`
- `update_download_completed`
- `update_installed`

### Deploy (I1-ENTERPRISE-DEPLOY)

- `deploy_silent_start`
- `deploy_policy_applied`
- `deploy_completed`

---

## 5. Event Schema

```json
{
  "ts": "2026-02-03T21:14:11.221Z",
  "session_id": "a7f3…",
  "level": "security",
  "event": "policy_denied",
  "actor": "ai_agent",
  "details": {
    "capability": "fs.write",
    "path": "/etc/passwd",
    "policy": "enterprise_default"
  },
  "hash": "prev+this"
}
```

---

## 6. Tamper Resistance

- append-only
- hash-chain
- optional: external signer, enterprise log forwarder (Splunk / ELK)

Удаление / правка: ❌ запрещено. Только export + archive.

---

## 7. Runtime Integration

### Где пишется

- `run_agent_loop`
- `command_router`
- `policy_engine`
- `sandbox`
- `installer`

### Когда

- **до действия**
- **после результата**
- **при отказе**

---

## 8. UI / UX

### Пользователь

- Timeline (read-only)
- Фильтр: AI / Security / Installer
- Кнопка **Explain this step**

### Enterprise

- Export: JSONL, signed ZIP
- Redaction (PII-safe)

---

## 9. AI Explainability

Explain ≠ Chain of Thought.

Используем: события, policy reason, rule name, model + role.

❌ без внутренних размышлений модели

---

## 10. Структура в репозитории

```
audit/
├── events.rs
├── writer.rs
├── reader.rs
├── chain.rs
├── export.rs
└── README.md
```

Связь: расширяет `crates/backend_core/src/audit.rs` (E6).

---

## 11. Критерий готовности

- события типизированы
- есть hash-chain
- audit связан с policy
- installer тоже логируется
- можно отдать аудитору

---

## См. также

- `docs/BLOCK_E6_SPEC.md` — базовая модель
- `docs/BLOCK_I1_SECURITY_SPEC.md` — security events
- `docs/BLOCK_I1_INSTALLER_SPEC.md` — installer audit
- `docs/BLOCK_I1_LICENSING_SPEC.md` — license events
- `docs/BLOCK_I1_UPDATE_SYSTEM_SPEC.md` — update events
- `docs/BLOCK_I1_ENTERPRISE_DEPLOY_SPEC.md` — deploy events
- `.ai/audit.md` — AI guidance
