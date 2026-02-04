# I3-PLUGIN-SYSTEM — Extensions & AI Plugins

> Расширяемость. AI-плагины. Marketplace (перспектива).

---

## 0. Цель

Сделать KengaIDE **расширяемой**:

- плагины как у VS Code
- AI-плагины (дополнительные tools, провайдеры)
- основа для marketplace

---

## 1. Scope (минимальный)

### Phase 1

- Загрузка плагинов из `~/.kengaide/plugins/`
- Формат: manifest.json + WASM или native (Rust)
- API: регистрация tools, провайдеров
- Sandbox: плагин не имеет прямого FS / net без capability

### Phase 2

- Marketplace (опционально)
- Установка через UI
- Версионирование, зависимости

---

## 2. Plugin Manifest

```json
{
  "id": "my-ai-tool",
  "name": "My AI Tool",
  "version": "1.0.0",
  "main": "plugin.wasm",
  "capabilities": ["fs.read", "fs.write"],
  "tools": ["my_tool"],
  "providers": []
}
```

---

## 3. Integration Points

| Точка | Описание |
|-------|----------|
| Tools | Плагин регистрирует tool_call handlers |
| Providers | Плагин добавляет AiProvider |
| UI | Плагин добавляет пункты меню / панели |
| Policy | Плагин может расширять policy schema |

---

## 4. Security

- Плагин = изолированный контекст
- Capability-based доступ
- Policy может запрещать плагины
- Audit: plugin_loaded, plugin_tool_called

---

## 5. Связь блоков

| ← |
|---|
| I2-UX-USABILITY (продукт стабилен) |
| I1-SECURITY (capability model) |
| I2-POLICY (plugin policy) |

---

## 6. Вне scope I3 (первая итерация)

- Полноценный marketplace
- Плагины на Python (позже)
- Hot reload без restart

---

## См. также

- `docs/BLOCK_I3_PLUGIN_API_SPEC.md` — API для разработчиков плагинов
- `docs/BLOCK_I1_SECURITY_SPEC.md` — capability
- `docs/BLOCK_I2_POLICY_SPEC.md` — plugin policy
