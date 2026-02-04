# Сборка local_provider

Модель: **GigaChat3** из HuggingFace (ai-sage, MIT).
- По умолчанию: GigaChat3-10B-A1.8B (~10 ГБ) — десктоп
- Опционально: GigaChat3-702B-A36B (~170+ ГБ) — high-end

Требуется:
- CMake
- C++ компилятор (MSVC на Windows, gcc/clang на Linux)
- Rust toolchain

```bash
# Windows: установить Visual Studio Build Tools + CMake
# Linux: sudo apt install cmake build-essential
cargo build -p local_provider
```

Если CMake не установлен, IDE можно собрать без offline-модели:
```bash
cargo build -p kengaide --no-default-features
```
