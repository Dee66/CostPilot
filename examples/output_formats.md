# Machine-Parseable Output Examples

CostPilot provides machine-parseable JSON output for all commands using `--format json` or `-f json`.

## Scan Command

```bash
costpilot scan --plan tfplan.json --format json
```

**JSON Output:**
```json
{
  "summary": {
    "total_resources": 12,
    "analyzed": 12,
    "high_risk": 2,
    "medium_risk": 3,
    "low_risk": 1,
    "total_monthly_cost": 487.52,
    "cost_change": 245.30,
    "timestamp": "2025-12-07T10:30:00Z"
  },
  "detections": [
    {
      "severity": "high",
      "resource_id": "aws_nat_gateway.main",
      "resource_type": "aws_nat_gateway",
      "issue": "NAT Gateway cost accumulation",
      "monthly_cost": 32.85,
      "confidence": 0.95,
      "fix_snippet": "# Use VPC endpoints instead\nresource \"aws_vpc_endpoint\" \"s3\" {\n  vpc_id = aws_vpc.main.id\n  service_name = \"com.amazonaws.us-east-1.s3\"\n}"
    }
  ],
  "predictions": [
    {
      "resource_id": "aws_lambda_function.api",
      "predicted_monthly_cost": 125.40,
      "confidence": 0.87,
      "factors": {
        "invocations": 5000000,
        "memory_mb": 512,
        "duration_ms": 200
      }
    }
  ]
}
```

## Diff Command

```bash
costpilot diff --before old.json --after new.json --format json
```

**JSON Output:**
```json
{
  "baseline": {
    "total_monthly_cost": 1200.50,
    "resource_count": 45,
    "timestamp": "2025-12-01T10:00:00Z"
  },
  "proposed": {
    "total_monthly_cost": 1450.75,
    "resource_count": 48,
    "timestamp": "2025-12-07T10:30:00Z"
  },
  "delta": {
    "cost_change": 250.25,
    "cost_change_percent": 20.85,
    "resources_added": 5,
    "resources_removed": 2,
    "resources_changed": 8
  },
  "changed_resources": [
    {
      "resource_id": "aws_instance.web[0]",
      "resource_type": "aws_instance",
      "old_cost": 35.04,
      "new_cost": 70.08,
      "cost_delta": 35.04,
      "reason": "Instance type changed from t3.medium to t3.large"
    }
  ],
  "regressions": [
    {
      "type": "cost_spike",
      "severity": 8.5,
      "resource_id": "aws_rds_instance.main",
      "cost_increase": 180.00,
      "confidence": 0.92
    }
  ]
}
```

## SLO Check Command

```bash
costpilot slo check --format json
```

**JSON Output:**
```json
{
  "summary": {
    "total_slos": 5,
    "pass_count": 3,
    "warning_count": 1,
    "violation_count": 1,
    "no_data_count": 0,
    "compliance_rate": 0.80
  },
  "violations": [
    {
      "slo_name": "monthly_budget",
      "target": 5000.00,
      "actual": 5487.52,
      "status": "violation",
      "enforcement": "block",
      "message": "Monthly cost exceeds budget by $487.52 (9.75%)"
    }
  ],
  "warnings": [
    {
      "slo_name": "resource_efficiency",
      "target": 0.85,
      "actual": 0.78,
      "status": "warning",
      "enforcement": "warn",
      "message": "Resource utilization below target (78% vs 85%)"
    }
  ],
  "passed": [
    {
      "slo_name": "nat_gateway_limit",
      "target": 3,
      "actual": 2,
      "status": "pass"
    }
  ],
  "block_deployment": true
}
```

## Validate Command

```bash
costpilot validate policy.yaml --format json
```

**JSON Output:**
```json
{
  "valid": false,
  "file": "policy.yaml",
  "errors": [
    {
      "line": 15,
      "column": 8,
      "message": "Unknown field 'max_costs'",
      "severity": "error",
      "suggestion": "Did you mean 'max_cost'?"
    }
  ],
  "warnings": [
    {
      "line": 23,
      "message": "Duplicate rule ID 'nat-gateway-check'",
      "severity": "warning"
    }
  ],
  "schema_version": "1.0.0"
}
```

## Audit Query Command

```bash
costpilot audit query --author "jane@example.com" --format json
```

**JSON Output:**
```json
{
  "total_entries": 15,
  "filtered_entries": 3,
  "entries": [
    {
      "id": "audit_20251207_103045_a3f9",
      "timestamp": "2025-12-07T10:30:45Z",
      "action": "policy_override",
      "author": "jane@example.com",
      "resource": "aws_instance.web",
      "reason": "Emergency deployment - approved by CTO",
      "metadata": {
        "policy_id": "cost_limit_exceeded",
        "approval_id": "APR-2025-1207-003"
      }
    }
  ]
}
```

## Group Command

```bash
costpilot group --plan tfplan.json --by tag:environment --format json
```

**JSON Output:**
```json
{
  "grouping_key": "tag:environment",
  "total_cost": 2450.75,
  "groups": [
    {
      "name": "production",
      "resource_count": 32,
      "monthly_cost": 1820.50,
      "percentage": 74.28,
      "resources": [
        {
          "id": "aws_instance.web[0]",
          "type": "aws_instance",
          "cost": 70.08
        }
      ]
    },
    {
      "name": "staging",
      "resource_count": 12,
      "monthly_cost": 430.15,
      "percentage": 17.55
    },
    {
      "name": "dev",
      "resource_count": 8,
      "monthly_cost": 200.10,
      "percentage": 8.17
    }
  ]
}
```

## Map Command

```bash
costpilot map --plan tfplan.json --format json
```

**JSON Output:**
```json
{
  "nodes": [
    {
      "id": "aws_vpc.main",
      "type": "aws_vpc",
      "monthly_cost": 0.00,
      "dependencies": []
    },
    {
      "id": "aws_subnet.public",
      "type": "aws_subnet",
      "monthly_cost": 0.00,
      "dependencies": ["aws_vpc.main"]
    },
    {
      "id": "aws_nat_gateway.main",
      "type": "aws_nat_gateway",
      "monthly_cost": 32.85,
      "dependencies": ["aws_subnet.public"],
      "cost_propagation": {
        "direct_cost": 32.85,
        "downstream_cost": 0.00,
        "total_impact": 32.85
      }
    }
  ],
  "edges": [
    {
      "from": "aws_subnet.public",
      "to": "aws_vpc.main",
      "type": "depends_on"
    }
  ],
  "total_resources": 45,
  "total_cost": 2450.75
}
```

## Version Command

```bash
costpilot version --detailed --format json
```

**Note:** Version command does not support JSON format - use `--detailed` for more information:

```
CostPilot
Version: 1.0.0
Build: 1.0.0 (deterministic)
Features: Zero-IAM, WASM-safe, Offline
License: MIT
```

## Error Format

All commands return consistent error format in JSON mode:

```json
{
  "error": {
    "code": "PLAN_PARSE_ERROR",
    "message": "Failed to parse Terraform plan",
    "file": "tfplan.json",
    "line": 42,
    "details": "Invalid JSON: expected ',' or '}' at line 42 column 15"
  }
}
```

## Usage in CI/CD

Parse JSON output with `jq`:

```bash
# Check if deployment should be blocked
BLOCK=$(costpilot slo check --format json | jq -r '.block_deployment')
if [ "$BLOCK" = "true" ]; then
  echo "❌ SLO violations block deployment"
  exit 1
fi

# Extract total cost
COST=$(costpilot scan --plan tfplan.json --format json | jq -r '.summary.total_monthly_cost')
echo "Projected monthly cost: \$$COST"

# Count high-severity detections
HIGH_RISK=$(costpilot scan --plan tfplan.json --format json | jq '[.detections[] | select(.severity=="high")] | length')
echo "High-risk issues: $HIGH_RISK"
```

## Python Integration

```python
import subprocess
import json

# Run scan and parse output
result = subprocess.run(
    ['costpilot', 'scan', '--plan', 'tfplan.json', '--format', 'json'],
    capture_output=True,
    text=True,
    check=True
)

data = json.loads(result.stdout)
print(f"Total cost: ${data['summary']['total_monthly_cost']:.2f}")

for detection in data['detections']:
    if detection['severity'] == 'high':
        print(f"⚠️  {detection['resource_id']}: {detection['issue']}")
```
