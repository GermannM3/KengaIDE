# BLOCK I1-UX — Installer UX Specification (Enterprise-grade)

> Визуал уровня Cursor / JetBrains Toolbox. Готово для дизайнера / Cursor / v0 / Figma.

## Принципы

1. **Ощущение продукта, а не тулзы** — никаких серых виндовых визардов
2. **Zero documentation** — пользователь ничего не читает отдельно
3. **Control without fear** — enterprise-настройки есть, но не пугают
4. **One binary** — скачал → запустил → установил → работает

---

## Экран 0 — Bootstrap (Preflight, auto)

**Не UI, а системный шаг**, но с визуальной индикацией.

### Что происходит

- проверка OS
- проверка AVX2
- проверка RAM
- проверка диска
- определение online / offline
- поиск `models.bundle`

### UI

- fullscreen splash
- логотип KengaIDE
- статус-строка: `Checking system requirements…`

### Если проблема

- **Hard fail** → блокирующий экран
- **Soft warning** → пойдёт дальше, но с подсказкой

---

## Экран 1 — Welcome

### Цель

Создать ощущение серьёзного продукта с ИИ внутри.

### Layout

- слева — логотип + слоган
- справа — карточка установки

### Текст

**Заголовок:** Welcome to KengaIDE

**Подзаголовок:** AI-powered IDE with local and enterprise-grade models

### Кнопки

- **Install**
- Advanced options (link, не кнопка)

---

## Экран 2 — Installation Mode

### Цель

Выбор сценария без перегруза.

### Варианты (cards)

1. **Recommended**
   - System install
   - Local models
   - Auto-updates

2. **Portable**
   - No system changes
   - All data in one folder

3. **Enterprise / Custom**
   - Policies
   - Model control
   - Offline support

По умолчанию выбран **Recommended**.

---

## Экран 3 — Install Location

### Для System

- readonly: `C:\Program Files\KengaIDE`

### Для Portable

- folder picker

### Advanced

- data directory override (скрыто под toggle)

---

## Экран 4 — Policy (Enterprise-ready, но дружелюбно)

### Заголовок

Security & Network Policy

### Toggles

- ☐ Allow cloud models
- ☑ Allow local models only
- ☐ Enable audit logging (recommended)

### Если enterprise

- загрузка `policies.json`
- preview (read-only)

---

## Экран 5 — Model Selection (ключевой)

### Layout

Список моделей (cards):

| Card | Role | RAM | Disk | Badge |
|------|------|-----|------|-------|
| GigaChat 3 (Local) | General / Chat / Planning | 12 GB | ~20 GB | ✔ Recommended (RU compliant) |
| DeepSeek-Coder 6.7B | Coding | 8 GB | ~4 GB | ✔ Recommended for development |
| Custom / Later | Skip download | — | — | Configure later |

### Поведение

- чекбоксы
- live-оценка: `Disk: 24 GB`, `RAM required: 12 GB`

---

## Экран 6 — Download & Install

### UI

- progress bar
- per-model progress
- скорость
- ETA

### Текст

> Downloading models and setting up environment…

### Поведение

- Resume поддерживается
- Offline → unpack bundle
- Ошибка → retry / skip model

---

## Экран 7 — First Launch Setup

### Заголовок

First Launch Configuration

### Опции

- ☑ Set DeepSeek-Coder as coding model
- ☑ Enable Agent mode
- ☑ Create first project

---

## Экран 8 — Finish

### Текст

> KengaIDE is ready.

### Кнопки

- **Launch KengaIDE**
- Open documentation
- Exit

---

## UX-детали

### Визуальный стиль

- тёмный / нейтральный
- крупная типографика
- минимум текста
- иконки моделей
- микроанимации (progress, transitions)

### Тон

- уверенный
- спокойный
- без developer-шуток
