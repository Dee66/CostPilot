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
    
    let workflow_content = r#"name: CostPilot

on:
  pull_request:
    paths:
      - '**.tf'
      - '**.tfvars'
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
        working-directory: ./infrastructure

      - name: Terraform Plan
        run: terraform plan -out=tfplan.binary
        working-directory: ./infrastructure

      - name: Convert plan to JSON
        run: terraform show -json tfplan.binary > tfplan.json
        working-directory: ./infrastructure

      - name: Install CostPilot
        run: |
          # TODO: Replace with actual installation method
          # curl -sSL https://costpilot.dev/install.sh | bash
          echo "CostPilot installation placeholder"

      - name: Run CostPilot Scan
        id: costpilot
        run: |
          costpilot scan \
            --plan=infrastructure/tfplan.json \
            --explain \
            --autofix=snippet \
            --format=markdown \
            > cost-report.md
        continue-on-error: true

      - name: Comment PR
        uses: actions/github-script@v7
        if: github.event_name == 'pull_request'
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('cost-report.md', 'utf8');
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });

      - name: Check Cost Thresholds
        run: |
          costpilot scan \
            --plan=infrastructure/tfplan.json \
            --format=json \
            --fail-on-critical
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

terraform-plan:
  stage: validate
  image: hashicorp/terraform:1.6
  script:
    - cd infrastructure
    - terraform init
    - terraform plan -out=tfplan.binary
    - terraform show -json tfplan.binary > tfplan.json
  artifacts:
    paths:
      - infrastructure/tfplan.json
    expire_in: 1 day

costpilot-scan:
  stage: cost-analysis
  image: ubuntu:22.04
  dependencies:
    - terraform-plan
  script:
    - apt-get update && apt-get install -y curl
    # TODO: Replace with actual installation method
    # - curl -sSL https://costpilot.dev/install.sh | bash
    - echo "CostPilot installation placeholder"
    - costpilot scan --plan=infrastructure/tfplan.json --explain --autofix=snippet > cost-report.md
  artifacts:
    reports:
      markdown: cost-report.md
  allow_failure: true
"#
    };

    write_file(&ci_path, ci_content)?;
    Ok(())
}

/// Generate example policy file
fn generate_example_policy(costpilot_dir: &Path) -> Result<(), String> {
    let policy_path = costpilot_dir.join("policy.yml");
    
    let policy_content = r#"# CostPilot Policy Configuration (Phase 2)
version: 1.0.0

# Budget policies
budgets:
  # Global monthly budget
  global:
    monthly_limit: 1000  # USD
    warning_threshold: 0.8  # 80%
  
  # Per-module budgets
  modules:
    - name: compute
      monthly_limit: 500
    - name: storage
      monthly_limit: 200
    - name: networking
      monthly_limit: 100

# Resource policies
resources:
  # NAT Gateway limits
  nat_gateways:
    max_count: 2
    require_justification: true
  
  # EC2 instance policies
  ec2_instances:
    allowed_families:
      - t3
      - t3a
      - m5
      - c5
    max_size: xlarge  # Require approval for 2xlarge+
  
  # S3 bucket policies
  s3_buckets:
    require_lifecycle_rules: true
    require_encryption: true
  
  # Lambda function policies
  lambda_functions:
    require_concurrency_limit: true
    max_memory_mb: 3008
  
  # DynamoDB table policies
  dynamodb_tables:
    prefer_provisioned: true  # Warn on PAY_PER_REQUEST

# SLO policies (Phase 2)
slos:
  - name: monthly_cost_under_budget
    type: cost_threshold
    value: 1000
    severity: CRITICAL
  
  - name: no_critical_anti_patterns
    type: pattern_detection
    patterns:
      - nat_gateway_overuse
      - unbounded_lambda_concurrency
    severity: HIGH

# Enforcement
enforcement:
  mode: advisory  # advisory, blocking
  block_on_critical: false  # Set true to fail CI on critical issues
"#;

    write_file(&policy_path, policy_content)?;
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
