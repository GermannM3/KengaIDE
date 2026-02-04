# BLOCK E5 — Enterprise Model Orchestration

## Цель
Предсказуемая, управляемая, объяснимая работа ИИ: разные модели → разные роли. Модель НЕ выбирает себя сама.

## Pipeline
```
User Action → Task Classifier → Role Resolver → Policy Check → Model Provider → Execution
```

## Компоненты

### 1. model_roles.json
- Путь: `~/.kengaide/model_roles.json` или `project_root/.kengaide/model_roles.json`
- Роли: chat, coding, planning, analysis, documentation
- Дефолт создаётся при первом запуске

### 2. TaskClassifier (rule-based)
- Вход: AiMode + user message
- Выход: TaskRole
- Без LLM, детерминированный

### 3. RoleResolver
- role → model_id из model_roles.json

### 4. ProviderSelector
- model_id → provider (по model_id() в AiProvider)
- Fallback: preferred_provider_id, затем любой доступный

### 5. Structured logging
- `runtime.log`: mode, role, model, policy
- `agent.log`: role, model при старте

### 6. UI
- Бейдж: `role · model_id`
- События: ai_model_selected, agent_progress (model_selected)
