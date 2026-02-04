# I2-POLICY — AI Governance & IDE Policy

> Управляемый, объяснимый и ограничиваемый AI. Enterprise-ready.

---

## 0. Цель

Сделать **управляемый, объяснимый и ограничиваемый ИИ**, пригодный для:

- enterprise
- regulated сред
- командной работы
- продаж

AI **никогда не «сам по себе»** — он всегда под политикой.

---

## 1. Policy Model

### Иерархия (приоритет сверху вниз)

```
CLI flags
↓
project/.kengaide/policy.json
↓
~/.kengaide/policy.json
↓
installer defaults (config/policies.json)
```

### Формат policy.json

```json
{
  "ai": {
    "allow_tools": ["read_file", "apply_patch", "list_files", "create_file"],
    "deny_tools": ["delete_file"],
    "max_steps": 15,
    "allowed_roles": ["coding", "analysis", "chat"],
    "allowed_models": ["deepseek-coder", "gigachat3"],
    "network": false
  },
  "files": {
    "read_only": ["security/", "licensing/", ".ai/"]
  },
  "audit": {
    "level": "full"
  }
}
```

### Связь с I1-SECURITY

`policy.json` — runtime-слой. `policies.json` (I1-SECURITY) — installer/enterprise override. Policy engine объединяет оба.

---

## 2. Policy Engine (runtime)

### Проверяется **перед каждым действием**

- tool_call
- model selection
- file write
- network access

### При нарушении

- действие **не выполняется**
- пишется audit event (`policy_denied`)
- агенту возвращается **policy violation message**

AI **не может обойти policy** — проверка в runtime, не только в prompt.

---

## 3. Policy → Prompt Injection

Политика **встраивается в system prompt**:

```
You are operating under a strict policy:
- You may not use tools: delete_file
- You may only modify files outside security/, licensing/, .ai/
- If blocked, explain why and stop.
```

Эффекты:

- снижает «галлюцинации»
- стабильное поведение
- объяснимо для аудиторов

---

## 4. Policy UI

### Где

- **Settings → AI Policy**
- **Project Settings → AI Policy** (project override)

### Что

- чекбоксы (allow_tools, deny_tools)
- dropdown (allowed_models, audit level)
- presets:
  - Developer — максимум свободы
  - Enterprise — ограничения
  - Experimental — новые фичи
  - Locked — минимум

Все изменения → **audit event** (`policy_updated`).

---

## 5. UI отражает ограничения

- Policy запрещает delete_file → пункт меню серый / скрыт
- Policy запрещает cloud → cloud models скрыты
- Policy enterprise → темы только allowlist (если задано)

UI **не ломается**, а адаптируется.

---

## 6. Структура в репозитории

```
policy/
├── model.rs       # Policy struct, merge hierarchy
├── engine.rs      # check_tool, check_model, check_file
├── loader.rs      # load from paths
└── README.md
```

Интеграция: `crates/ai_runtime` вызывает policy engine перед tool_call.

---

## 7. Audit Integration

События:

- `policy_loaded`
- `policy_updated`
- `policy_denied` (уже в I1-AUDIT-LOGGING)

---

## 8. Связь блоков

| ← | → |
|---|---|
| I1-SECURITY | I2-MENU, I2-THEME |
| I1-AUDIT-LOGGING | — |

---

## См. также

- `docs/BLOCK_I1_SECURITY_SPEC.md` — policies.json, capability
- `docs/BLOCK_I2_MENU_SPEC.md` — AI Policy в меню
- `docs/BLOCK_I2_THEME_SPEC.md` — policy theme_allowlist
- `docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md` — policy events
- `.ai/policy.md` — AI guidance (policy engine = protected)
