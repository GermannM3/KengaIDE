#!/usr/bin/env node
/** Запуск собранного KengaIDE (работает в Git Bash, PowerShell, cmd). */
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const exe = path.join(__dirname, "..", "target", "release", "kengaide.exe");
const proc = spawn(exe, [], { detached: true, stdio: "ignore", cwd: path.join(__dirname, "..") });
proc.unref();
