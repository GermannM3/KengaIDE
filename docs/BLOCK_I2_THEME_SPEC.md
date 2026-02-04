# I2-THEME — VSCode-compatible Themes

> Темы как у нормальной IDE. VSCode JSON — легально, MIT.

---

## 0. Цель

Поддержка тем VSCode-формата: Light / Dark, semantic tokens, custom themes.

---

## 1. Формат тем

VSCode theme:

- JSON
- MIT лицензия
- Можно легально использовать

### Структура (упрощённо)

```json
{
  "name": "Dark+",
  "type": "dark",
  "colors": {
    "editor.background": "#1e1e1e",
    "editor.foreground": "#d4d4d4"
  },
  "tokenColors": [...]
}
```

---

## 2. Theme Engine

### Компоненты

| Компонент | Назначение |
|-----------|------------|
| theme_loader.rs | Загрузка .json, parse |
| theme_registry | Список доступных тем |
| active_theme | Текущая тема (persist) |

### Пути хранения

```
~/.kengaide/themes/
project/.kengaide/themes/
```

Bundled: 2–3 базовые темы (Light, Dark, High Contrast).

---

## 3. Monaco Integration

Monaco:

- `editor.defineTheme()`
- `editor.setTheme()`
- Поддержка `colors` и `tokenColors`

---

## 4. Theme UI

### Где

- **View → Theme** (подменю)
- Light
- Dark
- High Contrast
- Custom… (выбор из ~/.kengaide/themes/)

### Функции

- Preview
- Live reload при смене
- Import VSCode theme (.json)

### Позже

- Marketplace (I2-Plugins)

---

## 5. Policy Integration

- Policy enterprise + `theme_allowlist` → только разрешённые темы
- По умолчанию — все

---

## 6. Структура в репозитории

```
themes/
├── loader.rs
├── registry.rs
├── bundled/
│   ├── light.json
│   ├── dark.json
│   └── high-contrast.json
└── README.md
```

Или: модуль в `crates/backend_core` или отдельный `crates/theme`.

---

## 7. active_theme.json

```
~/.kengaide/active_theme.json
```

```json
{
  "id": "dark",
  "path": "~/.kengaide/themes/dark.json"
}
```

При старте — загрузить и применить.

---

## 8. Связь блоков

| ← | → |
|---|---|
| I2-MENU | View → Theme |
| I2-POLICY | theme_allowlist |

---

## См. также

- `docs/BLOCK_I2_MENU_SPEC.md` — View → Theme
- `docs/BLOCK_I2_POLICY_SPEC.md` — theme restrictions
- Monaco Editor API: `defineTheme`, `setTheme`
