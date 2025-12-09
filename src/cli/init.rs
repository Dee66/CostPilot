// CLI init command - generate project configuration and CI templates

use std::fs;
use std::path::Path;
use colored::Colorize;

/// Initialize CostPilot in a project
pub fn init(directory: &str, ci_provider: &str) -> Result<(), String> {
    let project_dir = Path::new(directory);
    
    println!("{}", "🚀 Initializing CostPilot...".bold().cyan());
    
    // Create .costpilot directory
    let costpilot_dir = project_dir.join(".costpilot");
    create_directory(&costpilot_dir)?;
    
    // Generate configuration file
    generate_config_file(&costpilot_dir)?;
    
    // Generate CI templates based on provider
    match ci_provider {
        "github" => generate_github_action(&project_dir)?,
        "gitlab" => generate_gitlab_ci(&project_dir)?,
        "none" => {
            println!("{}", "  Skipping CI template generation".dimmed());
        }
        _ => return Err(format!("Unsupported CI provider: {}", ci_provider)),
    }
    
    // Generate example policy file
    generate_example_policy(&costpilot_dir)?;
    
    // Generate .gitignore entries
    generate_gitignore_entries(&project_dir)?;
    
    println!("\n{}", "✅ CostPilot initialized successfully!".bold().green());
    println!("\n{}", "Next steps:".bold());
    println!("  1. Review .costpilot/config.yml");
    println!("  2. Customize .costpilot/policy.yml if needed");
    if ci_provider != "none" {
        println!("  3. Commit the CI configuration file");
        println!("  4. CostPilot will run automatically on pull requests");
    } else {
        println!("  3. Run 'costpilot scan' manually to analyze costs");
    }
    
    Ok(())
}

/// Create directory if it doesn't exist
fn create_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        println!("  {} {}", "✓".green(), format!("Created {}", path.display()).dimmed());
    } else {
        println!("  {} {}", "→".yellow(), format!("{} already exists", path.display()).dimmed());
    }
    Ok(())
}

/// Generate configuration file
fn generate_config_file(costpilot_dir: &Path) -> Result<(), String> {
    let config_path = costpilot_dir.join("config.yml");
    
    let config_content = r#"# CostPilot Configuration
version: 1.0.0

# Detection settings
detection:
  enabled: true
  severity_threshold: LOW  # LOW, MEDIUM, HIGH, CRITICAL
  
  # Anti-pattern detection
  anti_patterns:
    - nat_gateway_overuse
    - overprovisioned_ec2
    - s3_missing_lifecycle
    - unbounded_lambda_concurrency
    - dynamodb_pay_per_request_default

# Prediction settings
prediction:
  enabled: true
  confidence_threshold: 0.5  # 0.0 - 1.0
  
  # Cost thresholds
  thresholds:
    warning: 100   # Monthly cost in USD
    critical: 500  # Monthly cost in USD

# Explain settings
explain:
  enabled: true
  max_patterns: 5  # Top N patterns to explain
  include_assumptions: true
  include_calculation_steps: true

# Autofix settings
autofix:
  mode: snippet  # snippet, patch (Pro), drift-safe (Beta)
  enabled: true
  require_approval: true  # For patch mode

# Policy evaluation
policy:
  enabled: false  # Enable in Phase 2
  policy_file: .costpilot/policy.yml

# Reporting
reporting:
  format: markdown  # markdown, json, text
  show_summary: true
  show_details: true
  show_snippets: true

# Zero-IAM enforcement
security:
  enforce_zero_iam: true
  enforce_wasm_sandbox: true
  max_file_size_mb: 20
  sandbox_memory_mb: 256
  sandbox_timeout_ms: 2000
"#;

    write_file(&config_path, config_content)?;
    Ok(())
}

/// Generate GitHub Action workflow
fn generate_github_action(project_dir: &Path) -> Result<(), String> {
    let workflows_dir = project_dir.join(".github").join("workflows");
    create_directory(&workflows_dir)?;
    
    let workflow_path = workflows_dir.join("costpilot.yml");
    
    let workflow_content = r#"name: CostPilot Cost Analysis

on:
  pull_request:
    paths:
      - '**.tf'
      - '**.tfvars'
      - 'terraform/**'
      - 'infrastructure/**'
      - '.github/workflows/costpilot.yml'

permissions:
  contents: read
  pull-requests: write

jobs:
  cost-analysis:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Terraform Init
        run: terraform init
        working-directory: ./terraform

      - name: Terraform Plan
        run: |
          terraform plan -out=plan.tfplan
          terraform show -json plan.tfplan > plan.json
        working-directory: ./terraform

      - name: Run CostPilot Analysis
        uses: Dee66/CostPilot@v1
        with:
          terraform_plan: terraform/plan.json
          policy_file: .costpilot/policy.yml
          baseline_file: .costpilot/baseline.json
          mode: all
          fail_on_regression: true
          fail_on_policy: true
          comment_pr: true

      - name: Upload Cost Report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: cost-analysis-report
          path: |
            terraform/plan.json
            cost-report.md
          retention-days: 30
"#;

    write_file(&workflow_path, workflow_content)?;
    Ok(())
}

/// Generate GitLab CI configuration
fn generate_gitlab_ci(project_dir: &Path) -> Result<(), String> {
    let ci_path = project_dir.join(".gitlab-ci.yml");
    
    // Check if file exists and append instead of overwrite
    let ci_content = if ci_path.exists() {
        println!("  {} .gitlab-ci.yml already exists, append CostPilot stage manually", "⚠".yellow());
        return Ok(());
    } else {
        r#"# GitLab CI Configuration with CostPilot

stages:
  - validate
  - cost-analysis
  - deploy

terraform-plan:
  stage: validate
  image: hashicorp/terraform:1.6
  script:
    - cd terraform
    - terraform init
    - terraform plan -out=plan.tfplan
    - terraform show -json plan.tfplan > plan.json
  artifacts:
    paths:
      - terraform/plan.json
    expire_in: 1 day
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"

costpilot-analysis:
  stage: cost-analysis
  image: ubuntu:22.04
  dependencies:
    - terraform-plan
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -fsSL https://costpilot.dev/install.sh | bash
  script:
    - |
      costpilot analyze \
        --plan terraform/plan.json \
        --policy .costpilot/policy.yml \
        --baseline .costpilot/baseline.json \
        --mode all \
        --format markdown > cost-report.md
  artifacts:
    reports:
      markdown: cost-report.md
    paths:
      - cost-report.md
    expire_in: 30 days
  allow_failure: false
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
"#
    };

    write_file(&ci_path, ci_content)?;
    Ok(())
}

/// Generate example policy file
fn generate_example_policy(costpilot_dir: &Path) -> Result<(), String> {
    let policy_path = costpilot_dir.join("policy.yml");
    
    let policy_content = r#"# CostPilot Policy Configuration
version: "1.0"

# Budget policies
policies:
  - name: "Production Budget Limit"
    rule: "monthly_cost <= 5000"
    action: block
    severity: CRITICAL
    tags:
      - environment: production
    
  - name: "Development Budget Warning"
    rule: "monthly_cost > 1000"
    action: warn
    severity: MEDIUM
    tags:
      - environment: development
  
  - name: "Instance Type Restrictions"
    rule: "instance_type in ['t3.micro', 't3.small', 't3.medium', 't3.large']"
    action: require_approval
    severity: HIGH
    resources:
      - aws_instance
      - aws_autoscaling_group
  
  - name: "NAT Gateway Limit"
    rule: "resource_count <= 2"
    action: block
    severity: HIGH
    resources:
      - aws_nat_gateway
  
  - name: "Lambda Concurrency Limit"
    rule: "reserved_concurrent_executions != null"
    action: warn
    severity: MEDIUM
    resources:
      - aws_lambda_function
  
  - name: "S3 Lifecycle Rules Required"
    rule: "lifecycle_rule != null"
    action: warn
    severity: LOW
    resources:
      - aws_s3_bucket
  
  - name: "Cost Regression Threshold"
    rule: "cost_increase_percent <= 20"
    action: require_approval
    severity: HIGH

# Exemptions
exemptions:
  - id: EXP-001
    policy: "Production Budget Limit"
    resource: "module.analytics.aws_emr_cluster"
    justification: "Q4 data processing spike - temporary"
    expires_at: "2026-03-31"
    approved_by: "tech-lead@company.com"
    ticket_ref: "JIRA-1234"

# Approval workflows
approval_workflows:
  require_approval_for:
    - cost_increase_percent: 30  # Require approval for 30%+ increases
    - monthly_cost: 10000        # Require approval for $10k+ changes
    - instance_types:            # Require approval for large instances
        - "*.2xlarge"
        - "*.4xlarge"
        - "*.8xlarge"
  
  approvers:
    - email: "tech-lead@company.com"
      slack: "@tech-lead"
    - email: "finops@company.com"
      slack: "@finops-team"

# Baseline configuration
baseline:
  file: ".costpilot/baseline.json"
  auto_update: false  # Set true to auto-update on main branch
  regression_threshold: 10  # Percent increase to flag as regression

# SLO configuration
slo:
  file: ".costpilot/slo.json"
  check_on_pr: true
  fail_on_breach: true
"#;

    write_file(&policy_path, policy_content)?;
    
    // Also generate baseline.json template
    let baseline_path = costpilot_dir.join("baseline.json");
    let baseline_content = r#"{
  "version": "1.0",
  "timestamp": "2025-12-07T00:00:00Z",
  "total_monthly_cost": 0.0,
  "resources": {},
  "metadata": {
    "terraform_version": "1.6.0",
    "provider_versions": {}
  }
}
"#;
    write_file(&baseline_path, baseline_content)?;
    
    // Generate SLO template
    let slo_path = costpilot_dir.join("slo.json");
    let slo_content = r#"{
  "version": "1.0",
  "slos": [
    {
      "name": "Monthly Cost Budget",
      "target": 5000.0,
      "error_budget_percent": 10,
      "window_days": 30,
      "breach_action": "alert"
    },
    {
      "name": "Cost Stability",
      "target_volatility_percent": 15,
      "window_days": 7,
      "breach_action": "warn"
    }
  ]
}
"#;
    write_file(&slo_path, slo_content)?;
    
    Ok(())
}

/// Generate .gitignore entries
fn generate_gitignore_entries(project_dir: &Path) -> Result<(), String> {
    let gitignore_path = project_dir.join(".gitignore");
    
    let entries = "\n# CostPilot\n.costpilot/snapshots/\n.costpilot/cache/\ntfplan.json\ntfplan.binary\ncost-report.md\n";
    
    if gitignore_path.exists() {
        let existing = fs::read_to_string(&gitignore_path)
            .map_err(|e| format!("Failed to read .gitignore: {}", e))?;
        
        if !existing.contains("# CostPilot") {
            fs::write(&gitignore_path, format!("{}{}", existing, entries))
                .map_err(|e| format!("Failed to append to .gitignore: {}", e))?;
            println!("  {} Updated .gitignore", "✓".green());
        } else {
            println!("  {} .gitignore already contains CostPilot entries", "→".yellow());
        }
    } else {
        write_file(&gitignore_path, entries)?;
    }
    
    Ok(())
}

/// Write file and report status
fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        println!("  {} {} already exists (skipped)", "→".yellow(), path.display());
        return Ok(());
    }
    
    fs::write(path, content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    
    println!("  {} Created {}", "✓".green(), path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        let result = init(path, "none");
        assert!(result.is_ok());
        
        // Verify directory structure
        assert!(temp_dir.path().join(".costpilot").exists());
        assert!(temp_dir.path().join(".costpilot/config.yml").exists());
        assert!(temp_dir.path().join(".costpilot/policy.yml").exists());
        assert!(temp_dir.path().join(".gitignore").exists());
    }

    #[test]
    fn test_init_github_creates_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        let result = init(path, "github");
        assert!(result.is_ok());
        
        assert!(temp_dir.path().join(".github/workflows/costpilot.yml").exists());
    }

    #[test]
    fn test_init_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        // Run init twice
        assert!(init(path, "none").is_ok());
        assert!(init(path, "none").is_ok());
        
        // Files should still exist
        assert!(temp_dir.path().join(".costpilot/config.yml").exists());
    }
}
