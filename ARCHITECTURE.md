# Архитектура KengaIDE (v1.0)

Зафиксировано. Не отступать.

## Цель

IDE уровня VS Code / Cursor с встроенным AI-рантаймом, без жёсткой привязки к API, с возможностью офлайн-работы.

## Схема

```
┌─────────────────────────────┐
│         UI (Tauri)           │
│ React + TypeScript           │
│ Monaco Editor                │
└──────────────┬──────────────┘
               │ IPC (commands/events)
┌──────────────▼──────────────┐
│     Backend Core (Rust)       │
│  FS / Project / Git / Router │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│        AI Runtime            │
│  Providers / Context / RAG   │
└──────────────┬──────────────┘
               │
┌──────────────▼──────────────┐
│     Model Manager            │
│  Download / Load / Cache     │
└─────────────────────────────┘
```

## Структура crates

| Crate | Ответственность |
|-------|-----------------|
| `backend_core` | FS, Project, Git, Command Router, валидация |
| `context_manager` | Сбор контекста, лимиты, токенизация |
| `model_manager` | Загрузка, хранение, версии моделей |
| `ai_providers` | LocalProvider, ApiProvider, trait AiProvider |
| `ai_runtime` | Оркестрация провайдеров, промпты, режимы |
| `kengaide` (src-tauri) | Tauri app, IPC, state |

## Границы ответственности

| Модуль | Может | Не может |
|--------|-------|----------|
| UI | Показывать | Решать |
| Backend | Оркестрировать | Генерить текст |
| AI Runtime | Думать | Рисовать UI |
| Provider | Генерить | Знать контекст IDE |
| Model Manager | Хранить | Общаться с UI |

## Поток: "Explain code"

1. UI → событие → `AiRequest::Explain`
2. Backend → валидация → контекст
3. AI Runtime → выбор провайдера → промпт
4. Provider → inference
5. Ответ → по цепочке → UI отображает
