# I1-SECURITY — KengaIDE Security Model

> Audit-level спецификация. Формализованное доверие для CTO, SecOps, юристов.

---

## 1. Цель блока

Обеспечить **детерминированную, проверяемую и управляемую безопасность** KengaIDE:

- на этапе установки
- во время работы
- при использовании AI
- при enterprise-развёртывании

---

## 2. Модель угроз (Threat Model)

### Учитываем

| Угроза | Описание |
|--------|----------|
| Вредоносный плагин / модель | Подмена GGUF, инъекция в контекст |
| Prompt-injection через файлы | Код проекта как часть промпта |
| Неавторизованный сетевой доступ | AI или плагин → внешний API без политики |
| Утечка кода / данных через AI | Контекст уходит в облако |
| Подмена бинарей / моделей | MITM, неверный checksum |
| Нарушение политики компании | AI пишет код, когда запрещено |

### Не обещаем

- Защиту от пользователя с root
- Защиту от компрометированного ядра ОС

---

## 3. Security Zones

### Zone 0 — Installer (Immutable)

- AI-NO-TOUCH
- write-once
- проверка checksum
- sandbox bootstrap

### Zone 1 — Core Runtime (Protected)

- `agent.rs`
- `orchestration.rs`
- `command_router.rs`
- `audit.rs`
- `traits.rs`

**Запрещено:** runtime codegen, dynamic eval, hot-patch без подписи.

### Zone 2 — AI Runtime (Constrained)

- ограниченные capability
- только через Tool API
- policy-aware
- no direct FS / net

### Zone 3 — User Space

- проекты
- плагины
- скрипты
- UI-настройки

---

## 4. Capability-based Security

Любое действие = **capability**, а не «доверие».

Примеры:

```json
{
  "capability": "fs.write",
  "scope": "project",
  "path": "/workspace"
}
```

```json
{
  "capability": "net.outbound",
  "domains": ["api.openai.com"],
  "reason": "ai_completion"
}
```

AI **никогда** не имеет:

- `fs.root`
- `process.spawn`
- `net.raw`

---

## 5. Policy Engine

### Источник

- `policies.json`
- installer-generated
- enterprise-override

### Пример

```json
{
  "allow_cloud": false,
  "allowed_models": ["gigachat3", "deepseek-coder"],
  "audit_level": "full",
  "ai_write_code": true,
  "ai_apply_patch": false
}
```

Policy:

- применяется **до старта**
- immutable в runtime
- изменения → restart + audit

---

## 6. Network Security

По умолчанию:

- ❌ outbound = deny
- ❌ inbound = deny

Разрешается только:

- явно
- доменно
- логируемо

TLS: pinned certs (enterprise), MITM-safe режим.

---

## 7. File System Security

- Chroot / sandbox
- workspace ≠ system
- models ≠ runtime
- audit — append-only

AI:

- видит только virtual FS
- paths — маппинг, не реальные

---

## 8. Model Security

Каждая модель:

- в `manifest.json`
- SHA-256
- роль (`assistant`, `code`, `vision`)
- квоты RAM / VRAM

❌ неизвестная модель = отказ

---

## 9. Prompt & Context Security

- Context trimming
- Injection-guard
- System-prompt immutable
- User-prompt ≠ system

AI **не знает**:

- путей ОС
- внутренних ключей
- полной архитектуры

---

## 10. Audit Integration (E6)

Любое security-событие:

- policy deny
- sandbox violation
- capability reject

→ `AuditEvent::SecurityEvent`

---

## 11. Структура в репозитории

```
security/
├── policy.rs
├── capabilities.rs
├── sandbox.rs
├── network.rs
├── fs.rs
└── README.md
```

---

## 12. Критерий готовности

Блок I1-SECURITY готов, если:

- можно показать SecOps
- политики формализованы
- есть deny-by-default
- все зоны описаны
- AI физически ограничен

---

## См. также

- `docs/BLOCK_I2_POLICY_SPEC.md` — policy.json (runtime, расширяет policies)
- `docs/BLOCK_I1_SPEC.md` — installer, paths
- `docs/BLOCK_I1_INSTALLER_SPEC.md` — installer audit
- `docs/BLOCK_E6_SPEC.md` — audit (базовая модель)
- `docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md` — security-grade audit
- `.ai/security.md` — AI guidance
