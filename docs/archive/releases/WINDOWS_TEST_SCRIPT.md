# Windows Release Testing Script

**Binary**: costpilot-v1.0.1-windows-x86_64.exe
**Release**: v1.0.1
**Date**: 2026-01-10

---

## Prerequisites

- Windows 10 or later
- PowerShell 5.1 or later
- Internet connectivity

---

## Test Instructions

### 1. Download Binary

Open PowerShell and run:

```powershell
# Create test directory
New-Item -ItemType Directory -Path "$env:TEMP\costpilot-test" -Force
Set-Location "$env:TEMP\costpilot-test"

# Download binary
Invoke-WebRequest -Uri "https://github.com/Dee66/CostPilot/releases/download/v1.0.1/costpilot-v1.0.1-windows-x86_64.exe" -OutFile "costpilot.exe"

# Download checksums
Invoke-WebRequest -Uri "https://github.com/Dee66/CostPilot/releases/download/v1.0.1/SHA256SUMS.txt" -OutFile "SHA256SUMS.txt"
```

---

### 2. Verify Checksum

```powershell
# Compute checksum
$hash = (Get-FileHash -Path "costpilot.exe" -Algorithm SHA256).Hash.ToLower()

# Expected checksum
$expected = "15dd68b238b36d4ccf67971d49cebce911081b6b8fb29f68c58ea2e737d8fde9"

# Compare
if ($hash -eq $expected) {
    Write-Host "✅ Checksum verified" -ForegroundColor Green
} else {
    Write-Host "❌ Checksum mismatch!" -ForegroundColor Red
    Write-Host "Expected: $expected"
    Write-Host "Got:      $hash"
    exit 1
}
```

---

### 3. Test Version Command

```powershell
.\costpilot.exe --version
```

**Expected Output**:
```
costpilot 1.0.1 (Free)
```

**STOP Condition**: If output is NOT exactly `costpilot 1.0.1 (Free)`, STOP and report.

---

### 4. Test Help Command

```powershell
.\costpilot.exe --help
```

**Expected**: Should display help text with all subcommands, no errors.

**STOP Condition**: If any DLL or runtime errors occur, STOP and report.

---

### 5. Initialize Configuration

```powershell
# Create clean test directory
New-Item -ItemType Directory -Path "$env:TEMP\costpilot-smoke" -Force
Set-Location "$env:TEMP\costpilot-smoke"

# Run init
& "$env:TEMP\costpilot-test\costpilot.exe" init
```

**Expected**: Should create:
- `.costpilot/config.yml`
- `.costpilot/policy.yml`
- `.costpilot/baseline.json`
- `.costpilot/slo.json`
- `.github/workflows/costpilot.yml`

**Verify**:
```powershell
Get-ChildItem -Path ".costpilot" -Recurse
```

---

### 6. Create Test Terraform Plan

```powershell
# Create test plan
@"
{
  "format_version": "1.2",
  "terraform_version": "1.6.0",
  "resource_changes": [
    {
      "address": "aws_instance.web",
      "mode": "managed",
      "type": "aws_instance",
      "name": "web",
      "provider_name": "registry.terraform.io/hashicorp/aws",
      "change": {
        "actions": ["create"],
        "before": null,
        "after": {
          "ami": "ami-0c55b159cbfafe1f0",
          "instance_type": "t3.medium",
          "tags": {
            "Name": "web-server",
            "Environment": "production"
          }
        }
      }
    }
  ]
}
"@ | Out-File -FilePath "test_plan.json" -Encoding UTF8
```

---

### 7. Test Scan Command

```powershell
& "$env:TEMP\costpilot-test\costpilot.exe" scan --plan test_plan.json
```

**Expected Output**:
- Should detect 1 resource change
- Should estimate monthly cost (~$150.00)
- Should suggest Reserved Instance optimization
- No errors or crashes

---

### 8. Test Determinism

```powershell
# Run scan twice and save outputs
& "$env:TEMP\costpilot-test\costpilot.exe" scan --plan test_plan.json > scan1.txt 2>&1
& "$env:TEMP\costpilot-test\costpilot.exe" scan --plan test_plan.json > scan2.txt 2>&1

# Compare outputs
$diff = Compare-Object (Get-Content scan1.txt) (Get-Content scan2.txt)
if ($null -eq $diff) {
    Write-Host "✅ Deterministic output verified" -ForegroundColor Green
} else {
    Write-Host "❌ Outputs differ!" -ForegroundColor Red
    $diff
}
```

---

### 9. Test License Gating (Free Edition)

```powershell
& "$env:TEMP\costpilot-test\costpilot.exe" version
```

**Expected Output**: `costpilot 1.0.1 (Free)`

**STOP Condition**: If output shows anything other than "Free", STOP and report.

---

## Test Completion Checklist

After running all tests, verify:

- [ ] Binary downloaded successfully
- [ ] Checksum matched expected value
- [ ] `--version` shows `costpilot 1.0.1 (Free)`
- [ ] `--help` displays without errors
- [ ] No missing DLL errors
- [ ] `init` command creates all configuration files
- [ ] `scan` command analyzes Terraform plan successfully
- [ ] Scan output is deterministic (identical across runs)
- [ ] License gating shows Free edition correctly

---

## Report Results

If all tests pass, report:
```
✅ Windows v1.0.1 release verified
   - Binary: costpilot-v1.0.1-windows-x86_64.exe
   - Checksum: 15dd68b238b36d4ccf67971d49cebce911081b6b8fb29f68c58ea2e737d8fde9
   - Version: 1.0.1 (Free)
   - All smoke tests passed
```

If any test fails, report:
```
❌ Windows v1.0.1 test failure
   - Failed test: [test name]
   - Error: [error message]
   - Environment: [Windows version, PowerShell version]
```

---

## Cleanup

```powershell
# Remove test directories
Remove-Item -Path "$env:TEMP\costpilot-test" -Recurse -Force
Remove-Item -Path "$env:TEMP\costpilot-smoke" -Recurse -Force
```
