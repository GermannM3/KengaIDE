# Состояние AI в KengaIDE

## Текущая архитектура (streaming + cancel + agent)

- **AiProvider** (BLOCK 1): только streaming — `generate(request, options)` возвращает `Stream<Item = AiChunk>` (start → token* → end | error); `cancel(request_id)`; `is_available()`.
- **AiController** (BLOCK 2): `run_stream()` запускает генерацию, эмитит чанки в UI по событию `ai_chunk`; `cancel(request_id)` останавливает по id.
- **LocalProvider** (BLOCK 3): token-by-token через `InferenceEngine::generate_stream()` и канал; отмена через `AtomicBool` в inference loop.
- **UI** (BLOCK 6–7): вызов `ai_request_stream` → получение `request_id` → подписка на `ai_chunk`; start → loader, token → append, end/error → финал; кнопка STOP → `ai_cancel(request_id)`.
- **Команды**: `ai_request_stream`, `ai_cancel`, `ai_agent_request`; режимы Chat, Explain, Refactor, Generate, **Agent**.

### BLOCK 10 — Tooling + Agent Loop (сделано)

- **10.1 Tool API** (`crates/agent_tools`): `AiTool`-контракт — `ToolCall`, `ToolResult`; инструменты `create_file`, `read_file`, `list_files`, `update_file` (пути относительно project_root).
- **10.2 ToolExecutor**: выполняет вызовы в контексте проекта; проверка выхода пути за пределы project_root.
- **10.3 Agent system prompt**: `AGENT_SYSTEM_PROMPT` в `ai_runtime::agent` — «IDE agent, not chat»; формат вызова: `\`\`\`tool_call\n{ "name", "arguments" }\n\`\`\``.
- **10.4 Режим Agent**: только режим `Agent` подключает tools и agent loop; в остальных режимах — обычный чат/стриминг.
- **10.5 Agent loop** (`run_agent_loop`): generate → парсинг `parse_tool_call` из ответа → `ToolExecutor::run` → результат в контекст → повтор, пока модель выдаёт tool_call; иначе Done. События `agent_progress`: ToolCall, ToolResult, Done.
- **UI**: кнопка «Agent» → `ai_agent_request` → подписка на `agent_progress` (отображение вызовов и результата).

### BLOCK 11 — Diff / patch tooling (сделано)

- **apply_patch** (`crates/agent_tools`): инструмент `path`, `before`, `after` — точное вхождение `before` заменяется на `after` (один раз); 0 или >1 вхождений → структурированные ошибки (BeforeBlockNotFound, AmbiguousPatch, FileNotFound, IOError).
- **ToolExecutor**: зарегистрирован `apply_patch`; `update_file` оставлен, в режиме Agent не используется.
- **Agent prompt**: правило «никогда не перезаписывать файл целиком», только `apply_patch`; при ошибке патча — исправить контекст и повторить.
- **agent_progress**: события `patch_apply_started`, `patch_apply_success`, `patch_apply_error` (файл, успех/ошибка).
- **UI**: отображение Patch … / ✓ Patched / ✗ Patch error по событиям.

### Логотип и иконки

- **logo.jpg** в корне проекта (например `C:\KengaIDE\logo.jpg`): при сборке копируется в `public/logo.jpg` (сплеш) и генерируются PNG в `src-tauri/icons` (32x32, 128x128, 128x128@2x). Иконки окна и установщика берутся из `src-tauri/icons`.
- **Сплеш**: при запуске показывается `/logo.jpg` до готовности приложения; если файла нет — сплеш скрывается по onError.
- Рабочая директория проекта — только диск C (например `C:\KengaIDE`). Папка на D: не используется; при необходимости удалить старую копию на D: вручную. Артефакты в `target/` могут содержать старые пути — `cargo clean` при смене диска.

### BLOCK 12 — MCP integration (base) (сделано)

- **mcp_provider** (`crates/mcp_provider`): конфиг из `~/.kengaide/mcp.json` (формат как в Cursor: `mcpServers`, `url`, `headers`); если файла нет — MCP отключён.
- **MCP client**: HTTP POST, JSON-RPC 2.0, timeout 30s, retry 1; метод `call(method, params)`; ошибки типизированы (`McpError`).
- **MCPContextProvider**: `fetch_context(query)` — вызывает у каждого сервера метод `context/query` с `{"query": query}`; результат — `Vec<McpContextChunk>` (server, content). Ошибки логируются, недоступные серверы пропускаются.
- **Интеграция в agent**: перед первым ходом загружается конфиг MCP, запрашивается контекст по `user_message`; блок «Additional context from external knowledge systems: - [server]: …» вставляется между system prompt и «User: …»; лимит ~6000 символов (~25% контекста).
- **Agent rule**: в system prompt добавлено правило про MCP: «Use it to improve accuracy and decisions, but never blindly trust it».
- MCP отключён или недоступен → агент работает как раньше; при ошибках генерация не прерывается.

### BLOCK 13 — MCP Tools (сделано)

- **13.1 MCP Tool Discovery** (`mcp_provider`): метод `tools/list`, парсинг в `McpToolDescriptor` (server, name, description, input_schema). `McpClient::list_tools(server_name)`; кэш TTL 5 мин в `McpToolRegistry`.
- **13.2 Tool Registry**: `McpToolRegistry::list_all_tools()` — агрегация по всем серверам; имена `mcp::server::tool`. В промпт агента подмешиваются локальные инструменты + MCP (build_agent_system_prompt).
- **13.3 MCP Tool Execution**: при вызове `mcp::server::tool` — парсинг имени, маршрутизация в нужный MCP-клиент, вызов `tools/call` (name, arguments); результат в tool_result.
- **13.4 Agent Loop Upgrade**: агент может вызывать local tools и MCP tools; результат tool_result вставляется в контекст; агент продолжает генерацию. Лимит 8 вызовов инструментов на одно сообщение пользователя; при ошибке MCP — tool_result с ошибкой, генерация не прерывается.
- **13.5 System prompt**: добавлено правило про MCP tools и список MCP-инструментов в промпте.
- **13.6 Logging**: tracing для discovery, tool call, failures (без секретов).
- **13.7 Tests**: unit-тест парсинга `mcp::server::tool`; интеграционный тест `McpToolRegistry::from_config` с пустым конфигом.

---

## Ранее (проблема «ноль реакции»)

### 1. Backend: один раз на процесс

- **Проблема была:** при двух одновременных вызовах оба вызывали `InferenceEngine::load()` → `LlamaBackend::init()`, второй падал с `BackendAlreadyInitialized`.
- **Исправление:** в `crates/local_provider/src/provider.rs` в `ensure_engine()`:
  - после проверки кэша по read lock берётся **write lock**;
  - внутри write lock делается **double-check**: если движок уже есть (подгрузил другой поток) — возвращаем его;
  - загрузка модели выполняется только один раз, остальные ждут и получают уже загруженный движок.
- **Результат:** модель загружается один раз (в логе видна одна полная загрузка GGUF и тензоров).

### 2. Цепочка от UI до inference

- **Frontend** (`src/App.tsx`):
  - при нажатии на кнопку AI вызывается `inv("ai_request", { payload })`;
  - ответ ждётся через `await`; при успехе в `aiResponse` пишется `result.content` + мета (модель, latency, токены), при ошибке — `Error: ...`.
- **Tauri** (`src-tauri/src/commands.rs`):
  - команда `ai_request` принимает payload, валидирует через `state.router`, вызывает `state.ai_runtime.handle_request(...).await` и возвращает `AiResponse` или ошибку.
- **AiRuntime** (`crates/ai_runtime/src/runtime.rs`):
  - строит контекст, промпт, выбирает провайдер, вызывает `provider.generate(gen_request).await`.
- **LocalProvider** (`crates/local_provider/src/provider.rs`):
  - `generate()` вызывает `ensure_engine().await`, затем `tokio::task::spawn_blocking(|| engine.generate(prompt, max_tokens))` и по завершении возвращает `AiResponse`.
- **InferenceEngine** (`crates/local_provider/src/inference.rs`):
  - при каждом `generate()` создаётся **новый** `llama_context` (`model.new_context()`), выполняется предикт промпта и цикл генерации токенов до EOS или `max_tokens`; стриминга нет — ответ целиком в конце.

### 3. Что видно в терминале

- Один раз загружается модель (metadata, tokenizer, 414 tensors, ~10.57 GiB).
- Два раза подряд идёт блок «llama_context: constructing llama_context» и настройка KV cache / graph. Это значит, что **два раза** вызывается создание контекста:
  - либо два запроса (двойной клик / два вызова `generate()`),
  - либо один запрос и что-то ещё создаёт контекст — в текущем коде один `generate()` создаёт один контекст.
- После последней строки `llama_context: graph splits = 1` в логе **ничего больше нет**. То есть:
  - либо идёт цикл генерации токенов (decode) и он **ничего не логирует** — тогда просто долгое ожидание;
  - либо процесс зависает или падает без вывода.

---

## В чём проблема: «ноль реакции»

Пользователь не видит **никакой** обратной связи после нажатия «отправить» запрос к AI.

### Наиболее вероятные причины

1. **Долгая генерация на CPU без обратной связи**
   - Модель ~10B параметров, Q8_0, всё на CPU. Первый токен и полный ответ могут занимать **десятки секунд или минуты**.
   - Ответ отдаётся только **целиком** (нет стриминга), индикатора загрузки во время `inv("ai_request")` в UI **нет**.
   - Итог: кнопка нажата → `await inv("ai_request")` висит → на экране ничего не меняется → ощущение «ноль реакции».

2. **Два контекста подряд**
   - В логе два полных блока создания контекста. Если реально уходят два запроса (например, двойной клик или два вызова при одном действии), то два тяжёлых inference конкурируют за CPU и оба становятся ещё медленнее; оба ответа придут только когда оба завершатся.

3. **Таймаут или зависание**
   - Если `invoke` или Tauri/backend где-то ограничены по времени или блокируются — запрос может так и не вернуться, ошибка может не показываться или теряться.

4. **Ошибка после создания контекста**
   - Паника или ошибка внутри `decode`/sampler в `inference.rs` без логирования даст «тихое» падение или Err, который может не дойти до UI в читаемом виде.

---

## Что проверить / что можно сделать дальше (для инструкций)

1. **UI во время запроса**
   - Показывать индикатор загрузки / «Генерация…» сразу при нажатии и убирать при возврате `ai_request` (успех или ошибка).
   - Опционально: блокировать повторный клик, пока идёт запрос (чтобы не было двух контекстов подряд из-за двойного нажатия).

2. **Логирование на бэкенде**
   - В `LocalProvider::generate()`: логировать «generate started» и «generate finished» (и длительность).
   - В `InferenceEngine::generate()`: логировать после создания контекста, после первого decode, после первого сгенерированного токена (и раз в N токенов), при выходе по EOS или max_tokens. Так будет видно, зависает ли запрос на decode или доходит до конца.

3. **Стриминг (по желанию)**
   - Сейчас ответ отдаётся один раз в конце. Чтобы была «реакция» до завершения, нужен стриминг: провайдер/backend шлёт куски текста (события Tauri или другой канал), UI по мере поступления обновляет вывод.

4. **Таймауты и ошибки**
   - Проверить, что ошибка из `ai_request` (в т.ч. из `provider.generate` и `spawn_blocking`) всегда возвращается в invoke и отображается в `setAiResponse(\`Error: ${...}\`)`.
   - При необходимости добавить таймаут на стороне frontend (и/или backend) и показывать «Timeout» вместо бесконечного ожидания.

5. **Один запрос — один контекст**
   - Убедиться, что на одно действие пользователя вызывается один `ai_request` (нет двойной отправки из-за двойного клика или дублирования вызова `handleAiRequest`).

---

## Кратко

- **Сделано:** бэкенд инициализируется один раз, модель грузится, цепочка UI → Tauri → AiRuntime → LocalProvider → InferenceEngine работает; в логе видна загрузка модели и два создания контекста, после чего лог обрывается.
- **Проблема:** пользователь не видит реакции, потому что ответ приходит только в конце долгой генерации на CPU и в UI нет индикатора ожидания; возможны также два параллельных запроса и отсутствие явного логирования/обработки ошибок после создания контекста.

Когда будут инструкции по приоритетам (индикатор, логи, стриминг, таймауты), по этому документу можно точечно вносить изменения в код.
