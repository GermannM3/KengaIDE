# I1-ENTERPRISE-DEPLOY — Enterprise Deployment & Distribution

> MSI / DEB / policies для корпоративного развёртывания.

---

## 0. Позиционирование

**I1-ENTERPRISE-DEPLOY** — финальный блок цепочки I1.

Это:

- артефакты для корпоративного развёртывания
- silent install / unattended
- policy pre-configuration
- совместимость с GPO, SCCM, Intune, apt

---

## 1. Цели

| Цель | Описание |
|------|----------|
| Массовое развёртывание | silent install, no user interaction |
| Policy-first | policies.json до первого запуска |
| Air-gap | установка без интернета |
| Управляемость | GPO / apt / SCCM / Intune |

---

## 2. Артефакты

### Windows

| Формат | Назначение |
|--------|------------|
| MSI | GPO, SCCM, Intune, enterprise standard |
| NSIS (.exe) | standalone, с GUI |
| Offline bundle | MSI + models.bundle для air-gap |

### Linux

| Формат | Назначение |
|--------|------------|
| .deb | apt, dpkg, Debian/Ubuntu |
| .rpm | dnf/yum, RHEL/Fedora |
| AppImage | portable, без установки в систему |

---

## 3. Silent Install

### Windows (MSI)

```
msiexec /i KengaIDE-1.0.0.msi /quiet /norestart
```

Опциональные свойства:

- `INSTALLDIR` — путь установки
- `POLICY_FILE` — путь к policies.json
- `LICENSE_FILE` — путь к license.json
- `AUDIT_LEVEL` — off / basic / full / forensic

### Linux (.deb)

```
dpkg -i kengaide_1.0.0_amd64.deb
# или
apt install ./kengaide_1.0.0_amd64.deb
```

Pre-configure: `/etc/kengaide/policies.json` (если есть).

---

## 4. Policy Pre-configuration

При silent install:

- `policies.json` копируется в `config/`
- `license.json` (если enterprise) — в `config/`
- Источник: параметр установщика или дефолтный путь

Enterprise не видит экран выбора политики — всё задано.

---

## 5. Air-gap Deployment

1. Скачать offline bundle на машине с интернетом
2. Перенести на целевые машины (USB, внутренняя сеть)
3. Установить без сетевого доступа
4. Модели уже в bundle или отдельным архивом

---

## 6. GPO / SCCM / Intune (Windows)

### GPO

- Распространение MSI через Group Policy
- Assigned / Published
- Обновление через новую версию MSI

### SCCM

- Application / Package
- Detection: реестр или файл
- Dependency: WebView2 (если требуется)

### Intune

- Win32 app / MSI
- Requirements: OS version, architecture
- Supersedence для обновлений

---

## 7. apt / dnf (Linux)

### apt repository

- Добавить repo в sources.list
- `apt update && apt install kengaide`
- Автообновление: `apt upgrade` (если policy разрешает)

### dnf

- Аналогично для .rpm

---

## 8. Uninstall / Cleanup

### Windows

- MSI: `msiexec /x {ProductCode} /quiet`
- Удаление: Program Files, LocalAppData
- Оставить: audit logs (опционально, policy)

### Linux

- `apt remove kengaide` / `dpkg -r kengaide`
- `/opt/kengaide`, `~/.local/share/kengaide`

---

## 9. Audit Integration

События при enterprise deploy:

- `deploy_silent_start`
- `deploy_policy_applied`
- `deploy_completed`

Все с: source (msi/deb), policy_path, license_id (если есть).

---

## 10. AI Rules (.ai/enterprise_deploy.md)

AI:

- ❌ не меняет silent install logic
- ❌ не трогает GPO/SCCM/apt конфиги
- ✅ может документировать процесс
- ✅ может описывать сценарии развёртывания

---

## 11. Структура в репозитории

```
deploy/
├── msi/
│   └── wix/
├── deb/
│   └── control, rules
├── policies/
│   └── enterprise_default.json
└── README.md
```

---

## 12. Критерий готовности

- MSI для Windows
- .deb для Debian/Ubuntu
- Silent install работает
- Policy pre-configuration
- Документация для админа (краткая)

---

## 13. Связь блоков

| ← |
|---|
| I1-INSTALLER |
| I1-UPDATE-SYSTEM |
| I1-LICENSING |
| I1-SECURITY |

---

## См. также

- `docs/BLOCK_I1_SPEC.md` — пути, manifest
- `docs/BLOCK_I1_INSTALLER_SPEC.md` — installer
- `docs/BLOCK_I1_SECURITY_SPEC.md` — policies
- `docs/BLOCK_18_SPEC.md` — базовые форматы (MSI, NSIS, deb)
- `.ai/enterprise_deploy.md` — AI guidance
