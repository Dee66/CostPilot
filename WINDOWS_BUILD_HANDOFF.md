# Windows Build Handoff

**Commit**: 639e5475
**Date**: 2026-01-10

---

## Build Command

```powershell
git checkout 639e5475
cargo build --release --target x86_64-pc-windows-msvc
```

---

## Expected Output

**Path**: `target\x86_64-pc-windows-msvc\release\costpilot.exe`
**Size**: 9-12 MB

---

## Success Criteria

```powershell
.\target\x86_64-pc-windows-msvc\release\costpilot.exe --version
# Expected: costpilot 1.0.0
```

---

## STOP Conditions

- If build fails: STOP
- If `--version` does not output 1.0.0: STOP
- If binary size > 20 MB: STOP

---

## Embedded Keys (Verify Only)

**LICENSE**: db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df
**WASM**: 8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994

Do NOT modify. Do NOT regenerate.

---

## Requirements

- Rust 1.91.1+
- Windows 10+
- MSVC Build Tools

---

**No Linux work remains. Execute above commands only.**
