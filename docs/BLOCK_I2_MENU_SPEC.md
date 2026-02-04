# I2-MENU — IDE Menu Bar & Command Palette

> Нормальное IDE-меню. Без него — игрушка.

---

## 0. Цель

IDE выглядит как IDE: верхнее меню + Command Palette 1:1 как VS Code.

---

## 1. Menu Bar (Tauri Menu API)

Нативное меню через `tauri::Menu`, не кастомные dropdown.

### File

| Пункт | Действие |
|-------|----------|
| New Project | Открыть модал создания проекта |
| Open Folder | Диалог выбора папки |
| Save | Сохранить текущий файл |
| Save All | Сохранить все открытые |
| — | |
| Exit | Закрыть приложение |

### Edit

| Пункт | Действие |
|-------|----------|
| Undo | Monaco undo |
| Redo | Monaco redo |
| — | |
| Cut | Monaco cut |
| Copy | Monaco copy |
| Paste | Monaco paste |
| — | |
| Find | Monaco find |
| Replace | Monaco replace |

### View

| Пункт | Действие |
|-------|----------|
| Toggle Sidebar | Показать/скрыть дерево файлов |
| Split Editor | Включить split view |
| Command Palette | Ctrl+Shift+P |
| Fullscreen | Переключить fullscreen |
| Theme | Подменю: Light / Dark / Custom |
| — | |

### AI

| Пункт | Действие |
|-------|----------|
| Run Agent | Открыть диалог задачи агента |
| Stop Agent | Остановить текущего агента |
| AI Policy | Открыть настройки политики |
| Model Selector | Открыть выбор модели |
| Audit Session | Открыть последнюю сессию аудита |

### Tools

| Пункт | Действие |
|-------|----------|
| MCP Servers | Открыть папку mcp.json |
| Extensions | (будущий I2-Plugins) |
| Terminal | (будущий) |

### Help

| Пункт | Действие |
|-------|----------|
| Documentation | Открыть docs |
| Open Logs | Открыть папку логов |
| About | Версия, лицензия |

---

## 2. Policy-aware Menu

- Policy запрещает tool → пункт серый или скрыт
- Policy enterprise → AI Policy только read-only
- Меню **отражает ограничения**, не ломается

---

## 3. Command Palette (Ctrl+Shift+P)

- Те же команды, что и в меню
- Fuzzy search
- Меню = UI, palette = power-user
- 1:1 как VS Code по UX

---

## 4. Реализация

### Tauri

- `Menu::new()`, `MenuItem`, `Submenu`
- `AppHandle::set_menu()`
- `on_menu_event` для обработки кликов

### Связь с React

- Tauri menu event → `emit` в frontend
- Или: команды через `invoke`, меню вызывает те же команды

---

## 5. Структура

```
src-tauri/src/
├── menu.rs      # Построение меню
└── ...
```

---

## 6. Связь блоков

| ← | → |
|---|---|
| I2-POLICY | UI отражает policy |
| BLOCK 15 | Command Palette уже есть, расширить |

---

## См. также

- `docs/BLOCK_I2_POLICY_SPEC.md` — policy-aware menu
- `docs/BLOCK_I2_UX_POLISH_SPEC.md` — Top Menu polish, hover, disabled states
- `docs/BLOCK_I2_THEME_SPEC.md` — View → Theme
