# I3-PLUGIN-API — Plugin Development API

> Контракт для разработчиков плагинов. Tools, Providers, Lifecycle.

---

## 0. Цель

Определить **API**, который плагин использует для интеграции с KengaIDE:
tools, providers, lifecycle, capability.

---

## 1. Plugin Lifecycle

```
load → init → register → (runtime) → unload
```

| Фаза | Описание |
|------|----------|
| load | Загрузка manifest, main (WASM/native) |
| init | Вызов `plugin_init(ctx)` |
| register | Регистрация tools, providers |
| runtime | Обработка вызовов |
| unload | Вызов `plugin_unload()`, освобождение |

---

## 2. Plugin Context (передаётся в init)

```rust
pub struct PluginContext {
    pub project_root: PathBuf,
    pub register_tool: fn(ToolDescriptor),
    pub register_provider: fn(Arc<dyn AiProvider>),
    pub request_capability: fn(Capability) -> bool,
    pub emit_audit: fn(AuditEvent),
}
```

---

## 3. Tool Descriptor

```rust
pub struct ToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: JsonSchema,
    pub handler: fn(ToolCall) -> ToolResult,
}
```

Плагин регистрирует tool. Agent вызывает по имени. Handler выполняется в sandbox.

---

## 4. Provider Registration

Плагин может добавить `AiProvider` (например, свой API).

Требования:

- Реализация trait `AiProvider`
- `id`, `name`, `is_available`, `generate`, `model_id`
- Policy может запретить внешние провайдеры

---

## 5. Capability Request

Перед доступом к FS / net плагин вызывает:

```rust
ctx.request_capability(Capability::FsRead { path: "..." })
```

Возврат: `true` если policy разрешает, иначе `false`. Отказ → audit event.

---

## 6. Audit Integration

Плагин может эмитить события:

```rust
ctx.emit_audit(AuditEvent::PluginToolCalled {
    plugin_id: "...",
    tool: "...",
    success: true,
})
```

---

## 7. Host → Plugin Calls

| Вызов | Когда |
|-------|-------|
| `tool_handler(call)` | Agent вызвал tool плагина |
| `provider.generate(...)` | Runtime выбрал провайдер плагина |
| `plugin_unload()` | Выгрузка плагина |

---

## 8. WASM Sandbox (Phase 1)

- Плагин = WASM module
- Host предоставляет syscalls: capability, audit, project_root (read-only paths)
- Нет прямого доступа к std::fs, std::net

---

## 9. Native Plugins (Phase 2)

- Rust dynamic library (.dll / .so / .dylib)
- Тот же контракт
- Более широкие capability, но требует доверия

---

## 10. Versioning

API версионируется. Manifest указывает:

```json
{
  "api_version": "1.0",
  "min_host_version": "0.2.0"
}
```

Несовместимость → плагин не загружается.

---

## 11. Связь блоков

| ← |
|---|
| BLOCK_I3_PLUGIN_SPEC |
| BLOCK_I1_SECURITY_SPEC (capability) |
| BLOCK_I2_POLICY_SPEC (plugin policy) |

---

## См. также

- `docs/BLOCK_I3_PLUGIN_SPEC.md` — обзор
- `docs/BLOCK_I1_SECURITY_SPEC.md` — capability model
- `crates/agent_tools` — эталон ToolCall / ToolResult
