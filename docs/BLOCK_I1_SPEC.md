# BLOCK I1 — Enterprise Installer & Isolated Runtime

> Расширяет **I1-ARCH** (архитектура установщика). Техническая спецификация для реализации.

## Цель

> **Один файл → одна установка → изолированная среда → готовый IDE + AI без документации**

Installer = Runtime Manager, а не копировщик файлов. Он создаёт окружение, кладёт модели, настраивает политики, управляет обновлениями.

---

## 0. Якорь

- Пользователь **не думает** об окружении.
- Сисадмин **не пишет инструкции**.
- Zero-docs, zero-manual.

---

## 1. Общая схема

```
┌───────────────────┐
│ Installer GUI     │  ← один .exe / .AppImage (отдельное приложение)
└─────────┬─────────┘
          │
          ▼
┌──────────────────────────────┐
│ Bootstrap Runtime Layer      │  ← нативный Rust (в составе installer)
└─────────┬────────────────────┘
          │
          ▼
┌──────────────────────────────┐
│ Isolated Kenga Runtime       │
│  ├─ runtime/                 │
│  ├─ models/                  │
│  ├─ data/                    │
│  └─ config/                  │
└──────────────────────────────┘
```

**Installer** — отдельный бинарник, не часть KengaIDE. Запускается первым, выполняет установку, затем запускает KengaIDE.

---

## 2. Пути установки

### 2.1 Windows

| Тип | Путь |
|-----|------|
| Программа | `C:\Program Files\KengaIDE\` |
| Данные пользователя | `%LOCALAPPDATA%\KengaIDE\` |

```
C:\Program Files\KengaIDE\
├─ runtime\
│  ├─ kengaide.exe          # основной бинарник (Tauri bundle)
│  ├─ WebView2Loader.dll    # если используется
│  └─ resources\            # иконки, etc.
│
├─ models\                  # модели (manifest-driven)
│  ├─ gigachat3\
│  │   ├─ model.gguf
│  │   ├─ tokenizer.json    # опционально
│  │   └─ manifest.json
│  └─ deepseek-coder\
│      ├─ model.gguf
│      └─ manifest.json
│
└─ config\                  # системный конфиг (read-only для пользователя)
   ├─ installer.json       # метаданные установки
   └─ policies.json        # enterprise policy (если задана)

%LOCALAPPDATA%\KengaIDE\
├─ data\
│  ├─ audit\
│  ├─ logs\
│  └─ cache\
├─ ai_config.json
├─ model_roles.json
└─ mcp.json
```

### 2.2 Linux

| Тип | Путь |
|-----|------|
| Программа | `/opt/kengaide/` |
| Данные пользователя | `~/.local/share/kengaide/` |

```
/opt/kengaide/
├─ runtime/
│  ├─ kengaide
│  └─ lib/
├─ models/
└─ config/

~/.local/share/kengaide/
├─ data/
├─ ai_config.json
├─ model_roles.json
└─ mcp.json
```

### 2.3 Режим «установлен vs portable»

KengaIDE при старте проверяет:

1. **Переменная окружения** `KENGAIDE_INSTALL_ROOT` — если задана, использовать её как корень установки.
2. **Файл-маркер** рядом с exe: `runtime/installer.json` — если существует, считать установку «managed».
3. **Иначе** — portable/legacy: `~/.kengaide/`, `%LOCALAPPDATA%\kengaide\` (текущее поведение).

```rust
// Псевдокод резолва
fn resolve_install_root() -> Option<PathBuf> {
    env::var("KENGAIDE_INSTALL_ROOT").ok().map(PathBuf::from)
        .or_else(|| exe_dir().and_then(|d| {
            let marker = d.join("..").join("config").join("installer.json");
            if marker.exists() { Some(d.join("..").canonicalize().ok()?) } else { None }
        }))
}
```

---

## 3. installer.json (метаданные установки)

Формат:

```json
{
  "version": "1.0.0",
  "installed_at": "2025-02-01T12:00:00Z",
  "installer_version": "1.0.0",
  "install_mode": "system",
  "install_root": "C:\\Program Files\\KengaIDE",
  "data_root": "C:\\Users\\...\\AppData\\Local\\KengaIDE",
  "policy_mode": "hybrid",
  "models_installed": ["gigachat3", "deepseek-coder"],
  "created_by": "installer-1.0.0"
}
```

| Поле | Описание |
|------|----------|
| version | версия KengaIDE |
| installed_at | ISO 8601 |
| install_mode | `system` \| `portable` |
| install_root | абсолютный путь к программе |
| data_root | абсолютный путь к данным пользователя |
| policy_mode | `offline` \| `hybrid` \| `cloud` |
| models_installed | список model_id |
| created_by | версия installer (support / migration / repair) |

---

## 4. Model manifest.json

Каждая модель в `models/{model_id}/` должна содержать `manifest.json`:

```json
{
  "id": "deepseek-coder",
  "version": "1.0",
  "role": ["coding", "analysis"],
  "quant": "Q4_K_M",
  "ram_required_mb": 8192,
  "gpu_optional": true,
  "license": "apache-2.0",
  "files": [
    {
      "path": "model.gguf",
      "url": "https://huggingface.co/TheBloke/deepseek-coder-6.7B-instruct-GGUF/resolve/main/deepseek-coder-6.7b-instruct.Q4_K_M.gguf",
      "sha256": "abc123..."
    }
  ]
}
```

| Поле | Тип | Описание |
|------|-----|----------|
| id | string | model_id (gigachat3, deepseek-coder) |
| version | string | версия модели |
| role | string[] | coding, analysis, chat, refactor |
| quant | string | Q4_K_M, Q8_0, etc. |
| ram_required_mb | number | минимум RAM |
| gpu_optional | bool | работает без GPU |
| license | string | SPDX |
| files | array | список файлов с url и sha256 |

Installer **читает manifest**, не хардкодит URL и пути.

---

## 5. policies.json (Enterprise)

Опционально. Кладётся в `config/policies.json` при установке с enterprise-флагом.

```json
{
  "allow_cloud": false,
  "allow_external_mcp": false,
  "allowed_models": ["gigachat3", "deepseek-coder"],
  "audit_level": "full",
  "telemetry": false
}
```

KengaIDE при старте проверяет этот файл и ограничивает поведение.

---

## 6. State Machine установщика

```
[Start]
   │
   ▼
┌─────────────┐
│ Preflight   │  OS, CPU (AVX), RAM, Disk, GPU
└──────┬──────┘
       │ OK
       ▼
┌─────────────┐
│ Policy      │  Offline / Hybrid / Cloud, Telemetry, Allowed models
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ ModelSel    │  Checkbox: GigaChat, DeepSeek, (опционально third-party)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Download    │  checksum, resume, progress, offline fallback
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Runtime     │  распаковать бинарники, прописать paths, создать config
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ FirstLaunch │  smoke-test, model load test, запуск IDE
└──────┬──────┘
       │
       ▼
[End]
```

### Переходы при ошибках

| State | OnSuccess | OnFailure | Retryable |
|-------|------------|-----------|-----------|
| Preflight | → Policy | показать сообщение, выход | — |
| Policy | → ModelSel | — | — |
| ModelSel | → Download | — | — |
| Download | → Runtime | retry, offline bundle, отмена | да |
| Runtime | → FirstLaunch | rollback, выход | — |
| FirstLaunch | → End | показать ошибку, Exit | — |

---

## 7. Preflight — минимальные требования

| Параметр | Минимум |
|----------|---------|
| OS | Windows 10+ / Ubuntu 20.04+ |
| CPU | x86_64, AVX2 |
| RAM | 8 GB (для DeepSeek), 12 GB (для GigaChat) |
| Disk | 2 GB (runtime) + сумма размеров выбранных моделей |
| GPU | опционально (CUDA 11+ / ROCm) |

Проверки:

- `std::arch::x86_64::__cpuid_count` для AVX2 (или аналог на Rust).
- `sysinfo` или `winapi` для RAM.
- Свободное место на диске — `fs2` или `std::fs::metadata`.

### Стратегия при отсутствии AVX2

**Не блокируем установку.** Выбранная стратегия:

1. **Soft warning** — показать предупреждение, разрешить продолжить.
2. **Ограничение моделей** — рекомендовать только smaller quant (DeepSeek Q4, не GigaChat Q8).
3. **Fallback** — при наличии CPU-only модели в manifest — предложить её вместо AVX2-оптимизированной.

Если ни одна модель не подходит — блокируем только выбор тяжёлых моделей, не саму установку.

---

## 8. Download — контракт

- **Resume**: HTTP Range requests.
- **Checksum**: SHA-256 после загрузки.
- **Progress**: callback `(bytes_done, bytes_total)`.
- **Offline**: если есть `models.bundle` рядом с installer — распаковать, пропустить download.

### models.bundle (offline)

- ZIP-архив: `models/gigachat3/`, `models/deepseek-coder/`.
- В корне: `manifest.json` с списком моделей и checksums.
- Installer при отсутствии сети ищет `models.bundle` в той же папке, что и exe.

---

## 9. Обновления (версионирование runtime)

**Не автообновление как в Chrome.**

```
runtime/
├─ 1.3.0/
├─ 1.4.0/
└─ current -> 1.4.0
```

- Новая версия ставится side-by-side.
- `current` — symlink на активную версию.
- Rollback: переключить symlink на предыдущую.
- Модели **не трогаются**, если не требуется новая версия.

В I1 делаем только структуру. Логика обновлений — I2.

---

## 10. Сборка installer

### 10.1 Компоненты

| Компонент | Описание |
|-----------|----------|
| `installer/` | Отдельный crate или подпапка в репо |
| Bootstrap | Rust, GUI (egui/iced/slint) или Tauri (минимальное окно) |
| KengaIDE bundle | Результат `tauri build` (exe + resources) |

### 10.2 Артефакты

| Платформа | Формат | Содержимое |
|-----------|--------|------------|
| Windows | `KengaIDE-Setup-1.0.0.exe` | NSIS/WiX: installer GUI + bootstrap |
| Linux | `KengaIDE-1.0.0.AppImage` | AppImage с installer + runtime |

### 10.3 Offline bundle

- `KengaIDE-Setup-1.0.0-offline.exe` = installer + models.bundle (встроенный или рядом).
- Размер: ~2 GB (с GigaChat + DeepSeek).

---

## 11. API Bootstrap → KengaIDE

Installer передаёт в KengaIDE:

1. **Путь установки** — через `KENGAIDE_INSTALL_ROOT` или `installer.json`.
2. **Список моделей** — через `ai_config.json` и `model_roles.json` (уже настроенные).
3. **Политики** — через `config/policies.json`.

KengaIDE при старте:

- Читает `install_root` из env или marker.
- Использует `{install_root}/models` как `models_dir`.
- Использует `%LOCALAPPDATA%\KengaIDE` / `~/.local/share/kengaide` для data.

---

## 12. Изменения в KengaIDE (backend)

### 12.1 Path resolution

Новый модуль `crates/backend_core/src/install_paths.rs`:

```rust
/// Возвращает корень установки, если приложение установлено через I1 installer.
pub fn install_root() -> Option<PathBuf>;

/// models_dir: install_root/models или default_models_dir()
pub fn models_dir() -> PathBuf;

/// data_dir: всегда user-specific (LocalAppData / ~/.local/share)
pub fn data_dir() -> PathBuf;
```

### 12.2 LocalProvider / ModelManager

- `LocalConfig::default_models_dir()` — заменить на `install_paths::models_dir()`.
- При `install_root().is_some()` — читать модели из `{install_root}/models/{model_id}/`.

### 12.3 Policies

- Новый модуль `crates/backend_core/src/policies.rs` (или в `ai_runtime`).
- Читать `config/policies.json` при старте.
- Ограничивать: cloud, MCP, telemetry, audit_level.

---

## 13. Вне scope I1

- Лицензирование, DRM, подписки — I2.
- Мастер-промпт для Cursor — I1-cursor.

**I1-UX** — экраны установщика описаны в `docs/BLOCK_I1_UX.md`.

---

## 14. Чеклист реализации

### Phase 1: Paths & Manifest

- [ ] `install_paths.rs`: `install_root()`, `models_dir()`, `data_dir()`
- [ ] Интеграция в `LocalConfig`, `ModelManager`, `AppState`
- [ ] Схема `manifest.json` для gigachat3, deepseek-coder
- [ ] `policies.rs`: чтение, применение ограничений

### Phase 2: Bootstrap (минимальный)

- [ ] Crate `installer` или `kengaide-installer`
- [ ] Preflight: OS, RAM, Disk, AVX2
- [ ] State machine (без GUI — CLI для проверки)
- [ ] Download с checksum, resume, progress

### Phase 3: Installer GUI

- [ ] GUI (egui/iced/slint или Tauri)
- [ ] Экраны: Preflight → Policy → ModelSel → Download → Runtime → Launch
- [ ] Сборка NSIS/WiX (Windows), AppImage (Linux)

### Phase 4: Offline & Enterprise

- [ ] models.bundle формат и распаковка
- [ ] policies.json при enterprise-установке
- [ ] Документация для сисадмина (минимальная)
