# CostPilot v1.0.0 Fork Build Plan

**Date:** 2026-01-10
**Objective:** Produce multi-platform release artifacts with ZERO cost using free GitHub Actions tier on forked repository.

---

## ⚠️ CRITICAL CONSTRAINTS

- **Fork account:** deonawsdev@gmail.com
- **Execute ONCE:** Single workflow run only
- **Free tier limit:** 2000 minutes/month
- **Expected usage:** ~240 minutes (leaves 1760 remaining)
- **Cost if run on paid account:** $3-5 per execution

---

## WORKFLOW AUDIT

### All Workflows (12 total)

| Workflow | Trigger | Runner Type | Cost Level | Status |
|----------|---------|-------------|------------|--------|
| `ci.yml` | ~~push/PR~~ **→ manual** | ubuntu-latest | Low | ✅ DISABLED |
| `macos-release.yml` | ~~tag~~ **→ manual** | macos-15-intel | **EXTREME** | ✅ DISABLED |
| **`release.yml`** | **workflow_dispatch** | **multi-platform matrix** | **HIGH** | **🎯 PRIMARY** |
| `core-build.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `core-artifact-release.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `pro-engine-build.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `chaos-testing.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `observability-testing.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `performance-testing.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `security-scanning.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `sustainability-testing.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |
| `deployment-orchestration.yml` | workflow_dispatch | ubuntu-latest | Low | Inert |

### Changes Made

1. **ci.yml:** Removed `push` and `pull_request` triggers → manual only
2. **macos-release.yml:** Removed `push.tags` trigger → manual only
3. **release.yml:** Added cost warning header and concurrency control

---

## 🎯 DESIGNATED BUILD WORKFLOW

**File:** `.github/workflows/release.yml`

### Platform Matrix

| Platform | Target | Runner | Est. Time | Cost Multiplier |
|----------|--------|--------|-----------|-----------------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `ubuntu-latest` | ~10 min | 1x (base) |
| macOS x86_64 | `x86_64-apple-darwin` | `macos-14` | ~80 min | 10x |
| macOS ARM64 | `aarch64-apple-darwin` | `macos-14` | ~80 min | 10x |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `windows-latest` | ~20 min | 2x |

**Total estimated time:** ~240 minutes
**Free tier impact:** 12% of monthly allowance (240/2000)

### Expected Artifacts (8 files)

1. `costpilot-1.0.0-linux-x86_64.tar.gz`
2. `costpilot-1.0.0-linux-x86_64.zip`
3. `costpilot-1.0.0-macos-x86_64.tar.gz`
4. `costpilot-1.0.0-macos-x86_64.zip`
5. `costpilot-1.0.0-macos-arm64.tar.gz`
6. `costpilot-1.0.0-macos-arm64.zip`
7. `costpilot-1.0.0-windows-x86_64.tar.gz`
8. `costpilot-1.0.0-windows-x86_64.zip`

Plus checksums: `sha256sum.txt`, `sha256sum.sig`

---

## GUARDRAILS IN PLACE

### Automatic Trigger Prevention

✅ No workflows trigger on:
- `push` events
- `pull_request` events
- `schedule` events
- Git `tags`

All workflows require **explicit `workflow_dispatch` manual trigger**.

### Concurrency Control

```yaml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false
```

- Prevents duplicate runs
- No accidental re-execution if triggered twice
- Single-flight guarantee per ref

### Cost Warning Header

```
⚠️ COST WARNING: This workflow uses macOS (10x) and Windows (2x) runners
Expected cost: ~240 CI minutes (~$3-5 depending on plan)
EXECUTE ONLY ONCE per version
```

---

## EXECUTION PLAN

### Phase 1: Fork Setup (Manual)

1. ✅ Create GitHub account: deonawsdev@gmail.com
2. Fork `Dee66/CostPilot` to new account
3. Make fork **public** (enables 2000 free minutes)
4. Verify `.github/workflows/` copied correctly

### Phase 2: Secrets Configuration (Manual)

Required secrets for artifact signing (optional but recommended):

- `SIGNING_SECRET` - Ed25519 private key for license signing
- `GPG_PRIVATE_KEY` - GPG key for checksum signing
- `GPG_PASSPHRASE` - GPG key passphrase
- `ED25519_PRIV` - Ed25519 private key (if different from SIGNING_SECRET)
- `PUBLISHER_PUBKEY` - Publisher public key

**If secrets missing:** Workflow will skip signing steps (non-blocking).

### Phase 3: Workflow Execution (ONE TIME ONLY)

1. Navigate to fork: `https://github.com/deonawsdev/CostPilot/actions`
2. Select workflow: **"Release"**
3. Click: **"Run workflow"** → **"Run workflow"**
4. Monitor progress (~4 hours total)
5. Download artifacts from workflow run page

### Phase 4: Artifact Transfer

1. Download all 8 artifacts from fork workflow
2. Upload to original repo: `Dee66/CostPilot` release v1.0.0
3. Delete fork (optional, saves quota)

---

## VERIFICATION CHECKLIST

Before triggering workflow on fork:

- [ ] Fork is **public** (not private)
- [ ] All automatic triggers disabled (verified above)
- [ ] Concurrency control in place
- [ ] Current branch matches desired release commit (main @ `02c35f75`)
- [ ] Free tier minutes available: >500 (safety buffer)

After workflow completes:

- [ ] All 4 platforms built successfully
- [ ] 8 artifact files generated (.tar.gz + .zip per platform)
- [ ] Checksums generated
- [ ] Binary version reports `1.0.0`
- [ ] Artifacts downloadable from workflow run

---

## COST ANALYSIS

### Free Tier Strategy

| Component | Minutes Used | Minutes Remaining |
|-----------|--------------|-------------------|
| Starting balance | 2000 | 2000 |
| Linux build (1×) | -10 | 1990 |
| macOS x86 build (1×) | -80 | 1910 |
| macOS ARM build (1×) | -80 | 1830 |
| Windows build (1×) | -20 | 1810 |
| **Safety buffer** | -50 | **1760** |

**Outcome:** 88% of free tier preserved for future use.

### Alternative Cost (if paid on main account)

- **Linux:** $0.008/min × 10 min = $0.08
- **macOS:** $0.08/min × 160 min = $12.80
- **Windows:** $0.016/min × 20 min = $0.32
- **Total:** ~$13.20 saved

---

## ROLLBACK PLAN

If workflow fails partway:

1. **DO NOT re-run entire workflow**
2. Check which platforms succeeded
3. Manually build failed platforms locally:
   - Linux: Already have artifacts ✅
   - Windows: Can build on free Azure Pipelines
   - macOS: Most expensive - last resort only

---

## POST-EXECUTION

After successful build:

1. Tag artifacts with git commit hash for traceability
2. Update release notes with build provenance
3. Test one artifact per platform before public announcement
4. Archive fork or delete to free up account quota

---

## CONFIRMATION

✅ **No automatic CI triggers remain**
✅ **Single workflow designated: release.yml**
✅ **Expected cost: $0 (free tier)**
✅ **Maximum executions: 1**
✅ **Guardrails in place: concurrency control, manual-only**

**Ready for fork-based execution.**

---

## STOP CONDITION MET

This plan is complete. **DO NOT trigger any workflows** until:

1. Fork is created and public
2. User confirms fork setup complete
3. Explicit approval given: "Execute release.yml on fork"
