# BLOCK 14 — Workspace & Project System ✅

## Цель
IDE понимает, где она и что за проект. Агент **не создаёт проект вручную** — только через `create_project(template)`.

## Компоненты

### 1. Workspace root
- Текущий `ProjectService` уже хранит `root` — это и есть workspace root
- Расширить: при открытии проверять/создавать `.kengaide/`

### 2. `.kengaide/config.json`
Структура в корне проекта:
```json
{
  "name": "my-project",
  "template": "rust",
  "created": "2025-02-01T12:00:00Z"
}
```
- Создаётся при `create_project` или при первом открытии
- Опционально: настройки проекта (пока минимально)

### 3. Шаблоны проектов
- `empty` — пустая папка + .kengaide/
- `rust` — Cargo.toml, src/main.rs
- `python` — requirements.txt, main.py
- `node` — package.json, index.js

### 4. API (backend_core)
- `get_workspace()` — текущий root (уже есть через ProjectService)
- `get_project_tree()` — дерево файлов (уже есть через get_project_tree command)
- `create_project(template, parent_path)` — создаёт проект из шаблона в parent_path, возвращает path

### 5. Agent
- Добавить инструмент `create_project(template: string, name?: string)` — создаёт проект в текущей папке или в выбранной
- В промпте: «Для создания нового проекта используй ТОЛЬКО create_project. Никогда не создавай Cargo.toml, package.json и т.п. вручную для нового проекта.»

### 6. Tauri commands
- `create_project` — (template, parent_dir, name?) → path
- При успехе — автоматически открыть проект
