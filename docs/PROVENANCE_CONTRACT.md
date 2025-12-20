# Heuristic Provenance Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

Every cost prediction in CostPilot must include **complete provenance information** tracing which heuristic (ID, version, confidence source) was used. This makes CostPilot's explain output fully auditable and builds trust in predictions.

---

## Required Fields

### Provenance Data Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeuristicProvenance {
    /// Unique identifier for the heuristic rule
    pub heuristic_id: String,

    /// Version of the heuristic (semantic versioning)
    pub heuristic_version: String,

    /// Source of confidence (Heuristic/Baseline/ColdStart/Historical)
    pub confidence_source: ConfidenceSource,

    /// Reason if fallback/cold-start was used
    pub fallback_reason: Option<FallbackReason>,

    /// Timestamp when heuristic was last updated
    pub heuristic_updated_at: String,  // ISO 8601

    /// SHA-256 hash of heuristic rule for verification
    pub provenance_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceSource {
    /// Primary heuristic from cost_heuristics.json
    Heuristic { file_version: String },

    /// From baseline file
    Baseline { baseline_version: String },

    /// Cold-start inference (no heuristic available)
    ColdStart { inference_method: String },

    /// Historical data analysis
    Historical { data_points: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FallbackReason {
    HeuristicMissing { resource_type: String },
    HeuristicStale { days_old: u32 },
    RegionNotSupported { region: String },
    InstanceTypeNotFound { instance_type: String },
    CustomResourceType { resource_type: String },
}
```

---

## Invariants

### 1. Explain Output Must Reference Provenance
```rust
// ❌ FORBIDDEN
pub struct Explanation {
    pub reasoning: Vec<ReasoningStep>,
    pub cost_estimate: f64,
    // Missing provenance!
}

// ✅ REQUIRED
pub struct Explanation {
    pub reasoning: Vec<ReasoningStep>,
    pub cost_estimate: f64,
    pub provenance: HeuristicProvenance,  // MUST be present
    pub confidence: f64,
}
```

**Enforcement:**
- Compiler enforces `provenance` field presence
- JSON schema validation requires provenance
- Tests verify provenance in all outputs

### 2. Provenance Hash Must Be Deterministic
```rust
use sha2::{Sha256, Digest};

pub fn calculate_provenance_hash(heuristic: &Heuristic) -> String {
    let mut hasher = Sha256::new();

    // Hash in deterministic order
    hasher.update(heuristic.id.as_bytes());
    hasher.update(heuristic.version.as_bytes());
    hasher.update(heuristic.resource_type.as_bytes());
    hasher.update(format!("{:.6}", heuristic.base_cost).as_bytes());

    // Sort and hash parameters
    let mut params: Vec<_> = heuristic.parameters.iter().collect();
    params.sort_by_key(|(k, _)| *k);
    for (key, value) in params {
        hasher.update(key.as_bytes());
        hasher.update(value.as_bytes());
    }

    format!("{:x}", hasher.finalize())
}
```

**Enforcement:**
- Provenance hash must be SHA-256
- Same heuristic always produces same hash
- Tests verify hash stability

### 3. Missing Provenance Is Hard Error
```rust
// ❌ FORBIDDEN - Silent fallback
pub fn predict_cost(resource: &Resource) -> Prediction {
    match load_heuristic(&resource.type_) {
        Some(h) => predict_with_heuristic(resource, h),
        None => guess_cost(resource),  // NO PROVENANCE!
    }
}

// ✅ REQUIRED - Provenance always tracked
pub fn predict_cost(resource: &Resource) -> Result<Prediction, Error> {
    let (prediction, provenance) = match load_heuristic(&resource.type_) {
        Some(h) => {
            let prov = HeuristicProvenance {
                heuristic_id: h.id.clone(),
                heuristic_version: h.version.clone(),
                confidence_source: ConfidenceSource::Heuristic {
                    file_version: h.file_version.clone(),
                },
                fallback_reason: None,
                heuristic_updated_at: h.updated_at.clone(),
                provenance_hash: calculate_provenance_hash(&h),
            };
            (predict_with_heuristic(resource, &h), prov)
        }
        None => {
            let prov = HeuristicProvenance {
                heuristic_id: format!("cold-start-{}", resource.type_),
                heuristic_version: "0.0.0".to_string(),
                confidence_source: ConfidenceSource::ColdStart {
                    inference_method: "conservative_estimate".to_string(),
                },
                fallback_reason: Some(FallbackReason::HeuristicMissing {
                    resource_type: resource.type_.clone(),
                }),
                heuristic_updated_at: chrono::Utc::now().to_rfc3339(),
                provenance_hash: calculate_provenance_hash(&default_heuristic()),
            };
            (cold_start_predict(resource), prov)
        }
    };

    Ok(Prediction {
        cost_estimate: prediction,
        provenance,
        confidence: calculate_confidence(&provenance),
    })
}
```

**Enforcement:**
- All prediction paths must return provenance
- Compile error if provenance not provided
- Runtime validation in debug builds

---

## Integration with Explain Engine

### Provenance in Reasoning Chain
```rust
pub struct ReasoningStep {
    pub step_type: StepType,
    pub input: InputValue,
    pub output: OutputValue,
    pub confidence_impact: ConfidenceImpact,
    pub provenance: HeuristicProvenance,  // Track provenance per step
}

pub fn explain_prediction(
    resource: &Resource,
    prediction: &Prediction,
) -> Explanation {
    let mut chain = ReasoningChainBuilder::new();

    // Step 1: Heuristic lookup
    chain.add_step(ReasoningStep {
        step_type: StepType::HeuristicLookup,
        input: InputValue::resource_type(&resource.type_),
        output: OutputValue::heuristic_found(&prediction.provenance.heuristic_id),
        confidence_impact: ConfidenceImpact::positive(0.9),
        provenance: prediction.provenance.clone(),
    });

    // Step 2: Base cost calculation
    chain.add_step(ReasoningStep {
        step_type: StepType::BaseCostCalculation,
        input: InputValue::price_per_hour(0.0416),
        output: OutputValue::monthly_cost(30.368),
        confidence_impact: ConfidenceImpact::neutral(),
        provenance: prediction.provenance.clone(),
    });

    // ... more steps

    Explanation {
        resource_address: resource.address.clone(),
        reasoning_chain: chain.build(),
        final_estimate: prediction.cost_estimate,
        confidence: prediction.confidence,
        provenance_summary: ProvenanceSummary {
            primary_heuristic: prediction.provenance.heuristic_id.clone(),
            heuristic_version: prediction.provenance.heuristic_version.clone(),
            confidence_source: prediction.provenance.confidence_source.clone(),
            fallback_used: prediction.provenance.fallback_reason.is_some(),
        },
    }
}
```

### JSON Output Example
```json
{
  "resource_address": "aws_instance.web",
  "resource_type": "aws_instance",
  "monthly_cost_estimate": 30.368,
  "confidence": 0.92,
  "provenance": {
    "heuristic_id": "aws_ec2_on_demand_t3_medium",
    "heuristic_version": "2.1.0",
    "confidence_source": {
      "type": "Heuristic",
      "file_version": "2024.11.0"
    },
    "fallback_reason": null,
    "heuristic_updated_at": "2024-11-15T00:00:00Z",
    "provenance_hash": "a3f5c9d2e1b4..."
  },
  "reasoning_chain": [
    {
      "step": 1,
      "type": "HeuristicLookup",
      "description": "Found heuristic for aws_instance (t3.medium)",
      "provenance_reference": "aws_ec2_on_demand_t3_medium@2.1.0"
    },
    {
      "step": 2,
      "type": "BaseCostCalculation",
      "description": "Base cost: $0.0416/hour × 730 hours/month",
      "calculation": "0.0416 * 730 = 30.368",
      "provenance_reference": "aws_ec2_on_demand_t3_medium@2.1.0"
    }
  ]
}
```

---

## Validation and Testing

### Provenance Validation Test
```rust
#[test]
fn test_all_predictions_have_provenance() {
    let plan = generate_mixed_terraform_plan(10, 5, 20);
    let results = scan_plan(&plan);

    for result in results.predictions {
        // Every prediction MUST have provenance
        assert!(result.provenance.is_some(),
            "Missing provenance for {}", result.resource_address);

        let prov = result.provenance.unwrap();

        // Validate required fields
        assert!(!prov.heuristic_id.is_empty());
        assert!(!prov.heuristic_version.is_empty());
        assert!(!prov.provenance_hash.is_empty());

        // Validate hash format (SHA-256 hex = 64 chars)
        assert_eq!(prov.provenance_hash.len(), 64);

        // Validate version format (semver)
        assert!(is_valid_semver(&prov.heuristic_version));
    }
}

#[test]
fn test_provenance_hash_determinism() {
    let heuristic = load_heuristic("aws_ec2_on_demand_t3_medium").unwrap();

    let hash1 = calculate_provenance_hash(&heuristic);
    let hash2 = calculate_provenance_hash(&heuristic);
    let hash3 = calculate_provenance_hash(&heuristic);

    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);
}

#[test]
fn test_cold_start_has_provenance() {
    let resource = Resource {
        type_: "aws_super_rare_resource".to_string(),
        address: "test.resource".to_string(),
        config: serde_json::json!({}),
    };

    let prediction = predict_cost(&resource).unwrap();

    // Even cold-start must have provenance
    assert_eq!(prediction.provenance.heuristic_id,
        "cold-start-aws_super_rare_resource");
    assert_eq!(prediction.provenance.heuristic_version, "0.0.0");

    match &prediction.provenance.confidence_source {
        ConfidenceSource::ColdStart { inference_method } => {
            assert_eq!(inference_method, "conservative_estimate");
        }
        _ => panic!("Expected ColdStart confidence source"),
    }

    assert!(prediction.provenance.fallback_reason.is_some());
}
```

### Provenance Schema Validation
```rust
#[test]
fn test_provenance_json_schema() {
    let prediction = predict_cost(&sample_resource()).unwrap();
    let json = serde_json::to_string(&prediction.provenance).unwrap();

    // Validate against JSON schema
    let schema = load_schema("schemas/provenance.json");
    assert!(validate_json(&json, &schema).is_ok());
}
```

---

## CLI Integration

### Show Provenance Information
```bash
# costpilot scan --show-provenance plan.json
{
  "resources": [...],
  "provenance_summary": {
    "heuristics_used": [
      {
        "id": "aws_ec2_on_demand_t3_medium",
        "version": "2.1.0",
        "count": 5,
        "total_cost": 151.84
      },
      {
        "id": "aws_rds_mysql_db_r5_large",
        "version": "2.0.0",
        "count": 2,
        "total_cost": 292.80
      }
    ],
    "cold_starts": [
      {
        "resource_type": "aws_super_rare_resource",
        "count": 1,
        "reason": "HeuristicMissing"
      }
    ],
    "heuristic_file_version": "2024.11.0",
    "heuristic_file_hash": "b7d8a6c4..."
  }
}
```

### Validate Heuristic Provenance
```bash
# costpilot heuristics validate-provenance
✅ All heuristics have valid provenance hashes
✅ All heuristics have semantic versions
✅ All heuristics have update timestamps
✅ Provenance file integrity verified

Heuristics summary:
- Total heuristics: 156
- Heuristic file version: 2024.11.0
- Last updated: 2024-11-15
- File hash: b7d8a6c4e3f2a1d9...
```

---

## Breaking This Contract

### Severity: CRITICAL

**You CANNOT:**
- ❌ Emit predictions without provenance
- ❌ Use non-deterministic provenance hashes
- ❌ Omit fallback_reason when using cold-start
- ❌ Skip provenance validation in tests

**You MUST:**
- ✅ Track provenance for every prediction
- ✅ Include provenance in explain output
- ✅ Calculate deterministic provenance hashes
- ✅ Document all confidence sources
- ✅ Expose provenance in CLI and JSON outputs

---

## Benefits

### For Users
- **Full auditability** - Know exactly which heuristic was used
- **Version tracking** - Understand if predictions change due to heuristic updates
- **Confidence transparency** - See why confidence is high or low
- **Debugging support** - Trace cold-start fallbacks

### For Developers
- **Explainability** - Makes explain engine fully transparent
- **Testing** - Verify correct heuristic selection
- **Analytics** - Track heuristic usage patterns
- **Quality control** - Detect stale or missing heuristics

### For Enterprise
- **Compliance** - Auditable prediction trail
- **Reproducibility** - Re-run with same heuristic version
- **Governance** - Track heuristic approval workflow
- **Cost attribution** - Understand prediction sources

---

## Version History

- **1.0.0** (2025-12-06) - Initial provenance contract

---

**This contract ensures CostPilot's predictions are always traceable and auditable.**
