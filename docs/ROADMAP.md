# KengaIDE — план развития до уровня Cursor / VS Code

## Текущее состояние (честно)

**Agent Runtime + Tooling Framework — готово.**  
**IDE Platform — ещё нет.**

### Что уже есть
- Агентный runtime, tool loop, MCP, diff/patch
- Локальные модели (GGUF), streaming, cancel
- Базовый UI: редактор, дерево файлов, панель AI

### Почему пока не Cursor
- Модель не гарантирует корректный tool_call
- Нет workspace awareness, project templates, bootstrap-логики
- Агент реагирующий, а не дирижирующий
- MCP без orchestration-слоя
- Нет workspace model, command palette, task system

---

## Must-have (без этого IDE не IDE)

1. **Workspace** — корень, `.kengaide/`, индекс, watcher
2. **Project bootstrap** — шаблоны (python, rust, node, empty), агент выбирает шаблон, не создаёт с нуля
3. **Command system** — Create project, Add file, Explain, Refactor, Fix errors
4. **Agent modes** — chat, agent, refactor, explain, search

---

## Roadmap по блокам

### BLOCK 14 — Workspace & Project System
**Цель:** IDE понимает, где она и что за проект.

- [x] Workspace root
- [x] `.kengaide/config.json`
- [x] Индекс файлов
- [x] API: `get_workspace()`, `get_project_tree()`, `create_project(template)`
- [x] **Агенту ЗАПРЕЩЕНО создавать проект вручную** — только через `create_project`

### BLOCK 15 — Command Palette (как VS Code)
**Цель:** пользователь управляет IDE командами, а не промптами.

- [x] `Ctrl+Shift+P`
- [x] Команды: New Project, Open Folder, Run Agent, MCP Settings, Add AI Provider, Switch Model
- [x] Агент вызывается **командой** (Run Agent → диалог ввода задачи)

### BLOCK 16 — Multi-Provider AI
- [x] `AiProvider`: OpenAI, GigaChat, Local (GGUF)
- [x] UI: добавить OpenAI по API key, выбрать провайдер (Command Palette)
- [x] Config: ~/.kengaide/ai_config.json, active_provider_id

### BLOCK 17 — Вторая локальная модель (обязательно)
- [x] DeepSeek-Coder 6.7B Instruct (Q4_K_M, ~4 ГБ)
- [x] GigaChat (RU) + DeepSeek-Coder (coding-first) — оба в runtime
- [x] Выбор провайдера в UI (Command Palette → Сменить провайдер), кнопка «Загрузить» для недоступных

### BLOCK 18 — Installer (Windows + Linux)
- [x] Windows: MSI + NSIS (targets: all), `npm run tauri:build`
- [x] Linux: AppImage + .deb (bundle.linux)
- [x] Первый запуск: Welcome-экран (RAM, CPU), кнопки «Загрузить модель» / «Пропустить»

### BLOCK 19 — IDE UX как у Cursor
- [x] File tree, tabs (вкладки с закрытием), split view (Ctrl+клик — открыть во втором)
- [x] Agent side panel, tool timeline (сворачиваемый список шагов)

### BLOCK 20 — Stability & Polish
- [x] 20.1 Agent Guardrails: max_steps, max_time, max_tokens, детекторы зацикливания, авто-stop
- [x] 20.2 Agent Reasoning Visibility: Thinking, иконки, кликабельные файлы в timeline
- [x] 20.3 Retry & Recovery: Повторить, С контекстом, Откатить патчи
- [x] 20.4 Model Quality Control: валидация ответа, авто-reprompt при длинных объяснениях
- [x] 20.5 First Run Experience: Новый проект / Открыть папку, рекомендация DeepSeek, Создать первый проект
- [x] 20.6 Telemetry: .kengaide/logs/agent.log, команда «Открыть папку логов»

### BLOCK E5 — Enterprise Model Orchestration
- [x] model_roles.json (role → model)
- [x] TaskClassifier (rule-based, без LLM)
- [x] RoleResolver + ProviderSelector по model_id
- [x] Structured logging (runtime.log, agent.log)
- [x] UI: бейдж role · model_id

### BLOCK E6 — Audit / Replay / Explainability
- [x] audit_dir, audit_events.jsonl (append-only)
- [x] Session meta (session_id, started_at, status)
- [x] Интеграция в agent: session_start, task_classified, model_selected, tool_call, tool_result, patch_applied, session_end
- [x] list_audit_sessions, get_audit_events, open_audit_folder
- [x] UI: SessionStarted, кнопка «Аудит», Command Palette

### BLOCK I1 — Enterprise Installer & Isolated Runtime
- [ ] install_paths.rs: install_root, models_dir, data_dir
- [ ] manifest.json для моделей, policies.json
- [ ] Installer: Preflight → Policy → ModelSel → Download → Runtime → Launch
- [ ] Offline: models.bundle, zero-docs
- [ ] См. docs/BLOCK_I1_SPEC.md, docs/BLOCK_I1_UX.md
- [x] I1-CURSOR: .ai/ — AI-инструментарий (system, architecture, rules, prompts)
- [ ] I1-INSTALLER: docs/BLOCK_I1_INSTALLER_SPEC.md — audit-level spec (8 экранов, audit, security)
- [ ] I1-SECURITY: docs/BLOCK_I1_SECURITY_SPEC.md — threat model, zones, capability, policy, audit
- [ ] I1-AUDIT-LOGGING: docs/BLOCK_I1_AUDIT_LOGGING_SPEC.md — levels, taxonomy, hash-chain, tamper resistance
- [ ] I1-LICENSING: docs/BLOCK_I1_LICENSING_SPEC.md — types, capabilities, binding, offline-first, audit
- [ ] I1-UPDATE-SYSTEM: docs/BLOCK_I1_UPDATE_SYSTEM_SPEC.md — side-by-side, rollback, policy, audit
- [ ] I1-ENTERPRISE-DEPLOY: docs/BLOCK_I1_ENTERPRISE_DEPLOY_SPEC.md — MSI, deb, silent install, GPO, apt

### BLOCK I2 — AI Policy & IDE Governance

- [ ] I2-POLICY: docs/BLOCK_I2_POLICY_SPEC.md — policy.json, engine, presets, UI
- [ ] I2-MENU: docs/BLOCK_I2_MENU_SPEC.md — Tauri Menu API, File/Edit/View/AI/Tools/Help
- [ ] I2-THEME: docs/BLOCK_I2_THEME_SPEC.md — VSCode themes, loader, View → Theme
- [ ] I2-UX-POLISH: docs/BLOCK_I2_UX_POLISH_SPEC.md — layout, status bar, timeline, empty states, typography
- [ ] I2-UX-USABILITY: docs/BLOCK_I2_UX_USABILITY_SPEC.md — сценарии, friction points, валидация

### BLOCK I3 — Plugin System

- [ ] I3-PLUGIN: docs/BLOCK_I3_PLUGIN_SPEC.md — extensions, AI-плагины, manifest, capability
- [ ] I3-PLUGIN-API: docs/BLOCK_I3_PLUGIN_API_SPEC.md — PluginContext, ToolDescriptor, lifecycle, WASM

---

## Навигация

Полный индекс спецификаций: [SPECS_INDEX.md](SPECS_INDEX.md)

---

## Приоритет выполнения

1. **workspace** (BLOCK 14) — фундамент
2. **command palette** (BLOCK 15) — управление
3. **models** (BLOCK 17) — качество агента
4. **installer** (BLOCK 18) — доставка пользователю
