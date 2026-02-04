# BLOCK E6 — Audit / Replay / Explainability

## Цель
Полная трассировка AI-вызовов. Воспроизводимость, объяснимость, аудит.

## Компоненты

### 1. Audit Session
- `session_id` — генерируется до первого токена
- `project_root/.kengaide/audit/` или `~/.kengaide/audit/`
- `{session_id}.jsonl` — append-only события
- `{session_id}_meta.json` — метаданные сессии

### 2. Audit Events (JSONL)
- session_start
- task_classified
- model_selected
- prompt_sent
- tool_call
- tool_result
- patch_applied
- error
- session_end

### 3. Интеграция
- Agent mode: полный аудит (обязательно)
- Streaming (chat/explain): опционально

### 4. Commands
- list_audit_sessions
- get_audit_events(session_id)
- open_audit_folder

### 5. UI
- SessionStarted → lastSessionId
- Кнопка «Аудит» после агента
- Command Palette: «Открыть папку аудита»
