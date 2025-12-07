# 🚀 CostPilot Implementation Checklist

## 📊 Overall Progress

<div role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="88" style="width:94%; background:#e6eef0; border-radius:8px; padding:6px; box-shadow: inset 0 1px 2px rgba(0,0,0,0.04);">
  <div style="width:88%; background:linear-gradient(90deg,#2e7d32,#4caf50,#66bb6a); color:#fff; padding:10px 12px; text-align:right; border-radius:6px; font-weight:700; transition:width 0.5s ease;">
    <span style="display:inline-block; background:rgba(0,0,0,0.12); padding:4px 8px; border-radius:999px; font-size:0.95em;">88% · 627/709</span>
  </div>
</div>

**Target:** Production-Ready Zero-IAM FinOps Engine

> **Version:** 1.0.2  
> **Disclaimer:** Comprehensive implementation checklist derived directly from the CostPilot Spec. ALL items must be completed or explicitly deferred according to roadmap. Deterministic, WASM-safe, zero-IAM execution is mandatory.

---

## 📋 Table of Contents

- [Phase 1: MVP (Trust Triangle)](#phase-1-mvp-trust-triangle) - Days 0-14
- [Phase 2: V1 (Governance & Graph)](#phase-2-v1-governance--graph) - Days 15-45
- [Phase 3: V2+ (Enterprise & Scale)](#phase-3-v2-enterprise--scale) - Days 45-120
- [Engines](#engines)
  - [Detection Engine](#detection-engine)
  - [Prediction Engine](#prediction-engine)
  - [Explain Engine](#explain-engine)
  - [Autofix Engine](#autofix-engine)
  - [Policy as Code](#policy-as-code)
  - [Baselines System](#baselines-system)
  - [SLO Engine](#slo-engine)
  - [Mapping Engine](#mapping-engine)
  - [Grouping Engine](#grouping-engine)
  - [Trend Engine](#trend-engine)
- [CLI](#cli)
- [Zero-IAM Security & WASM](#zero-iam-security--wasm)
- [Performance Budgets](#performance-budgets)
- [Testing & Acceptance](#testing--acceptance)
- [Quality Contracts](#quality-contracts)
- [Compatibility Enforcement](#compatibility-enforcement)
- [Config Validation Engine](#config-validation-engine)
- [Pricing & Licensing](#pricing--licensing)
- [Governance & Lifecycle](#governance--lifecycle)
- [Go-To-Market](#go-to-market)
- [Onboarding](#onboarding)
- [Release Pipeline](#release-pipeline)

---

## Phase 1: MVP (Trust Triangle)

**Timeline:** Days 0-14  
**Objectives:**
- Build the Trust Triangle: Detect → Predict → Explain
- Ship snippet-based autofix (deterministic, idempotent)
- Ship core CI integration (GitHub Action + PR comments)
- Validate zero-IAM, zero-network, deterministic core

### Must Have

- [x] CLI core and WASM runtime
- [x] Detection engine minimal
- [x] Prediction engine minimal
- [x] Explain engine (top 5 patterns)
- [x] Snippet autofix mode
- [x] CLI init and CI templates
- [x] Basic policy evaluation
- [x] Terraform plan JSON parsing
- [x] Zero-IAM security validation

---

## Phase 2: V1 (Governance & Graph)

**Timeline:** Days 15-45  
**Objectives:**
- Implement full Governance layer (PAC engine + metadata)
- Ship Graph-based mapping (Mermaid first)
- Ship Trend system (SVG priority, HTML fallback)
- Introduce Drift-Safe Autofix (Beta) + rollback support
- Implement Baselines system + Regression classifier

### Must Have

- [x] Policy as code - full metadata
- [x] Exemption workflow V1
- [x] Trend engine V1
- [x] Graph mapping V1
  - [x] Core graph types (GraphNode, GraphEdge, DependencyGraph)
  - [x] Graph builder with dependency inference
  - [x] Cycle detection algorithm (DFS-based)
  - [x] Mermaid diagram generation
  - [x] HTML wrapper with visualization
  - [x] Cost impact detection (cross-service analysis)
  - [x] Stable node IDs for deterministic output
- [x] Drift-safe autofix (Beta)
- [x] SLO engine V1
- [x] Artifact support (CDK, CloudFormation)
  - [x] Core artifact types (Artifact, ArtifactFormat, ArtifactResource)
  - [x] CloudFormation parser (JSON/YAML support)
  - [x] CDK parser (manifest.json + template parsing)
  - [x] Artifact normalizer (unified format conversion)
  - [x] Property mapping (PascalCase → snake_case, resource-specific)
  - [x] Intrinsic function resolution (Ref, GetAtt, Sub, Join)
  - [x] Format auto-detection
  - [x] Real-world examples (CFN web app, CDK serverless API)
  - [x] Comprehensive documentation (ARTIFACT_SUPPORT.md)
- [x] Zero-network policy enforcement
  - [x] Zero-network token system (compile-time guarantees)
  - [x] Policy engine zero-network methods
  - [x] Metadata engine zero-network methods
  - [x] Dependency validation (block network crates)
  - [x] Determinism validation (block non-deterministic ops)
  - [x] Zero-network runtime enforcement
  - [x] WASM-safe implementation
  - [x] Comprehensive test suite
  - [x] Documentation (ZERO_NETWORK.md)
- [x] Baselines system V1

---

## Phase 3: V2+ (Enterprise & Scale)

**Timeline:** Days 45-120  
**Objectives:**
- Enterprise suite: Burn alerts, Audit, SSO, Escrow
- Full policy lifecycle & approval workflow
- Advanced prediction (probabilistic model)
- Usage metering + PR-based billing hooks

### Must Have

- [x] SLO burn alerts
  - [x] Burn rate linear regression
  - [x] Time to breach prediction
  - [x] Risk classification (Low/Medium/High/Critical)
  - [x] Multi-SLO analysis
  - [x] Confidence scoring
- [x] Enterprise policy lifecycle and approval flow
  - [x] Policy lifecycle state machine (6 states)
  - [x] Multi-party approval workflow
  - [x] Role-based authorization
  - [x] Approval expiration tracking
  - [x] Policy version history and versioning
  - [x] Semantic versioning (major.minor.patch)
  - [x] SHA-256 content checksums
  - [x] Version diffing and rollback
  - [x] CLI integration (8 commands)
  - [x] Comprehensive documentation
- [x] Audit logs and tamper-proofing
  - [x] Immutable audit log with cryptographic chain
  - [x] Blockchain-style entry linking (SHA-256)
  - [x] Event type classification (15+ types)
  - [x] Severity levels (Low/Medium/High/Critical)
  - [x] Chain integrity verification
  - [x] Compliance framework support (SOC 2, ISO 27001, GDPR, HIPAA, PCI DSS)
  - [x] Automated compliance reporting
  - [x] Audit query builder with filters
  - [x] Multi-format export (JSON, NDJSON, CSV)
  - [x] CLI integration (6 commands)
  - [x] Comprehensive documentation
- [x] VS Code extension
  - [x] Extension manifest (package.json) with 6 commands, 12 settings
  - [x] TypeScript configuration and build setup
  - [x] Main extension entry point (extension.ts)
  - [x] CLI execution wrapper (cli.ts)
  - [x] Real-time diagnostics provider (diagnostics.ts)
  - [x] Quick fix code actions (codeActions.ts)
  - [x] Inline cost annotations (annotations.ts)
  - [x] Status bar integration (statusBar.ts)
  - [x] Tree view for issue navigation (treeView.ts)
  - [x] Webview panels for reports (webview.ts)
  - [x] Test suite and configuration
  - [x] Marketplace documentation (README.md)
  - [x] Comprehensive developer docs (VS_CODE_EXTENSION.md)
- [x] Advanced prediction model
  - [x] Probabilistic prediction engine (probabilistic.rs)
  - [x] Confidence intervals (P10/P50/P90/P99)
  - [x] Risk classification (Low/Moderate/High/VeryHigh)
  - [x] Uncertainty factor identification
  - [x] Multi-scenario analysis (Best/Expected/Worst/Catastrophic)
  - [x] Seasonality detection (seasonality.rs)
  - [x] Weekly and monthly pattern detection
  - [x] Seasonal adjustment factors
  - [x] Monte Carlo simulation (monte_carlo.rs)
  - [x] 10,000+ simulation runs with deterministic seeding
  - [x] VaR and CVaR calculation
  - [x] Distribution analysis and histogram generation
  - [x] Multiple uncertainty distributions (Normal, LogNormal, Uniform, Triangular)
  - [x] Comprehensive documentation (ADVANCED_PREDICTION.md)
- [x] Usage metering hooks
  - [x] Core usage meter (usage_meter.rs)
  - [x] Event tracking (8 event types)
  - [x] Attribution system (user/team/org/cost_center/project)
  - [x] Pricing models (4 tiers: Free/Solo/Pro/Enterprise)
  - [x] Team usage summaries with billing calculation
  - [x] PR-based billing tracker (pr_tracker.rs)
  - [x] PR lifecycle tracking (Open/Merged/Closed/Draft)
  - [x] ROI calculation (cost prevented / charge)
  - [x] CI/CD usage tracking
  - [x] Chargeback reporting (chargeback.rs)
  - [x] Organization-level reports
  - [x] Team/user/project cost allocation
  - [x] Invoice generation
  - [x] CSV export for external systems
  - [x] Module exports (mod.rs)
  - [x] CLI integration (usage.rs)
  - [x] 5 CLI commands (report/export/pr/chargeback/invoice)
  - [x] Comprehensive documentation (USAGE_METERING.md)
- [x] Software escrow process
  - [x] Escrow package builder (package.rs)
  - [x] Package metadata and versioning
  - [x] Source file inventory with checksums
  - [x] Build artifacts packaging
  - [x] Dependencies manifest
  - [x] Build instructions generation
  - [x] Verification system with integrity checks
  - [x] Release automation (release.rs)
  - [x] Git integration (commit hash, tags, branch)
  - [x] Automated deposit on releases
  - [x] Escrow agent API integration
  - [x] Deposit receipt generation
  - [x] Recovery orchestrator (recovery.rs)
  - [x] Prerequisites checking
  - [x] Source extraction
  - [x] Dependency installation
  - [x] Build from source
  - [x] Test execution
  - [x] Recovery playbook generation
  - [x] Module exports (mod.rs)
  - [x] CLI integration (escrow.rs)
  - [x] 6 CLI commands (create/verify/playbook/recover/configure/list)
  - [x] Comprehensive documentation (SOFTWARE_ESCROW.md)
- [x] Performance budgets hard limits
  - [x] Engine-specific budgets (6 engines: Prediction 300ms, Mapping 500ms, Autofix 400ms, Total Scan 2000ms, SLO 150ms, Policy 200ms)
  - [x] Memory limits (WASM 256MB, engines 64-512MB)
  - [x] File size limits per engine (5-20MB)
  - [x] Circuit breaker implementation (5 failure threshold, 3 success threshold, 60s timeout)
  - [x] Circuit breaker state machine (Closed/Open/HalfOpen)
  - [x] Performance tracker with budget enforcement
  - [x] Warning thresholds (80% of budget)
  - [x] Timeout actions (PartialResults/Error/CircuitBreak)
  - [x] Performance monitoring system
  - [x] Baseline management with percentiles (p50/p95/p99)
  - [x] Regression detection (20% threshold)
  - [x] Regression severity classification (Minor/Moderate/Severe/Critical)
  - [x] Performance history tracking (last 100 snapshots)
  - [x] Memory tracker with limit validation
  - [x] Budget violation reporting
  - [x] Performance metrics collection
  - [x] Module exports (mod.rs)
  - [x] CLI integration (performance.rs)
  - [x] 5 CLI commands (budgets/set-baseline/stats/check-regressions/history)
  - [x] Comprehensive documentation (PERFORMANCE_BUDGETS.md)

---

## Engines

### Detection Engine

- [x] Parse Terraform plan JSON
- [x] Parse CDK diff JSON (V1)
- [x] Parse CloudFormation changeset JSON (V1)
- [x] Normalize IaC to canonical form
- [x] Handle unknown values conservatively
- [x] Handle computed values conservatively
- [x] Handle taint/replace lifecycle impact
- [x] Detect cost smells
- [x] Detect cost risks
- [x] Detect cost explosions
- [x] Classify regression type and severity

#### Output Contract

- [x] Resource changes
- [x] Old cost estimate
- [x] New cost estimate
- [x] Estimated monthly delta
- [x] Severity score
- [x] Regression type
- [ ] Suggested fix snippet

---

### Prediction Engine

- [x] Load heuristics from cost_heuristics.json
  - [x] HeuristicsLoader with multiple search paths
  - [x] Automatic discovery (7 fallback locations)
  - [x] File validation (version, pricing ranges)
  - [x] JSON parsing with error handling
  - [x] Heuristics statistics collection
- [x] Conservative deterministic estimates
- [x] Cold start inference
  - [x] EC2 instance type estimation
  - [x] RDS instance class estimation
  - [x] Storage cost estimation
- [x] Predict confidence interval
- [x] Emit prediction metadata
- [x] Ensure prediction intervals never inverted
- [x] Enforce zero negative costs
- [x] CLI commands for heuristics management
  - [x] `heuristics stats` - Show statistics
  - [x] `heuristics paths` - Show search paths
  - [x] `heuristics validate` - Validate file
  - [x] `heuristics show` - Show service pricing

**Constraints:**
- WASM-safe: ✅
- Max latency: 300ms

---

### Explain Engine

- [x] Reference cost_heuristics file
- [x] Show stepwise reasoning (prediction)
  - [x] ReasoningChain data structure
  - [x] ReasoningStep with categories (8 types)
  - [x] ReasoningChainBuilder for fluent construction
  - [x] InputValue with source tracking
  - [x] OutputValue with units
  - [x] ConfidenceImpact tracking
  - [x] CostComponent breakdown
  - [x] Human-readable formatting
- [x] PredictionExplainer for all resource types
  - [x] EC2 instances with heuristic lookup
  - [x] RDS instances (MySQL/Postgres) with storage
  - [x] Lambda functions (requests + compute)
  - [x] DynamoDB tables (provisioned/on-demand)
  - [x] NAT Gateway (hourly + data transfer)
  - [x] Load Balancers (ALB with LCU)
  - [x] S3 buckets (storage + requests)
  - [x] Generic resources with cold-start
- [x] CLI commands
  - [x] `explain resource` - Explain specific resource
  - [x] `explain all` - Explain all resources in plan
  - [x] Verbose mode for full step-by-step
  - [x] Cost filtering and limiting
- [x] Include all assumptions
- [x] Integration with prediction engine
- [ ] Show stepwise reasoning (detection)
- [ ] Include regression type
- [ ] Include severity score
- [ ] Root cause classification (V2)

#### MVP Limitations - Top 5 Anti-Patterns

- [x] NAT gateway overuse
- [x] Overprovisioned EC2
- [x] S3 missing lifecycle
- [x] Unbounded Lambda concurrency
- [x] DynamoDB pay-per-request default

---

### Autofix Engine

#### Snippet Mode (MVP)

- [x] Generate snippet only
- [x] Include human rationale
- [x] Deterministic output
- [x] Idempotent output

#### Patch Mode (Pro Only)

- [x] Generate full patch diff
- [x] Unified diff format with @@ headers
- [x] Resource-specific patch logic (EC2, RDS, Lambda, DynamoDB, S3, NAT Gateway)
- [x] Cost savings estimation with reduction factors
- [x] Patch metadata (cost before/after, confidence, rationale)
- [x] Instance downsize recommendations
- [x] CLI integration (autofix --mode=patch)
- [x] Patch simulation pass required
- [x] Label as Beta (V1)

#### Drift-Safe Autofix (V1 Beta)

- [x] Verify no infra drift prior to patch
- [x] Generate rollback patch
- [x] Operation state tracking with snapshots
- [x] Six safety check types (drift, exists, hash, cost, policy, SLO)
- [x] Automatic rollback on failure
- [x] Drift detection with severity levels
- [x] Policy and SLO integration
- [x] Execution logging and audit trail
- [x] Configuration integrity verification
- [x] Documentation and examples

---

### Policy as Code

#### MVP Built-in Rules

- [x] NAT gateway limit
- [x] Module monthly budget
- [x] S3 lifecycle required
- [ ] Compute savings plan suggestion

#### Full Engine (V1)

- [x] YAML policy loader
- [x] Rule DSL parser
  - [x] PolicyRule structure with conditions and actions
  - [x] Condition types (ResourceType, ResourceAttribute, MonthlyCost, CostIncrease, ModulePath, Tag, ResourceCount, Expression)
  - [x] Operators (Equals, NotEquals, GreaterThan, LessThan, Contains, StartsWith, EndsWith, Matches, In, NotIn)
  - [x] Rule actions (Block, Warn, RequireApproval, SetBudget, TagResource)
  - [x] Rule severity levels (Critical, High, Medium, Low, Info)
  - [x] YAML and JSON parsing support
  - [x] Rule validation and error handling
  - [x] RuleEvaluator for condition evaluation
  - [x] EvaluationContext with resource data
  - [x] Multiple search paths for rule loading
  - [x] Directory scanning for rule files
  - [x] Rule statistics and reporting
  - [x] CLI commands (list, validate, test, stats, example)
- [x] Policy versioning metadata
- [x] Policy approval required flag
- [x] Policy violation counters
- [x] Exemption workflow schema check
- [x] Exemption expiration validation
- [x] Exemption wildcard matching

#### Full Metadata System (V1)

- [x] Rich policy metadata (ID, name, description, category, severity)
- [x] Ownership tracking (author, owner, team, contact, reviewers)
- [x] Lifecycle management (created, updated, effective dates, deprecation)
- [x] Status management (Draft, Active, Disabled, Deprecated, Archived)
- [x] Severity levels (Info, Warning, Error, Critical) with blocking
- [x] Policy categories (Budget, Resource, Security, Governance, SLO, etc.)
- [x] Tagging system for search and organization
- [x] Documentation links (runbooks, tickets, related)
- [x] Metrics tracking (evaluations, violations, exemptions, rates)
- [x] Version history and revision tracking
- [x] Policy repository with filtering and querying
- [x] Repository statistics and analytics
- [x] Bulk operations (activate, disable, archive)
- [x] Metadata policy engine with automatic metrics
- [x] Backward compatibility with legacy policies
- [x] Comprehensive documentation and examples

#### Enterprise Requirements

- [ ] Delegated ownership
- [ ] Approval workflows
- [ ] Audit trail

---

### Baselines System

- [x] Support baselines.json file
- [x] Record expected cost for module
- [x] Record last updated and justification
- [x] Integrate with regression classifier
- [x] Integrate with trend engine (via detect_baseline_violations)

---

### SLO Engine

#### MVP

- [x] Monthly cost SLO check
- [x] Module cost SLO check
- [x] Service budget SLO check
- [x] Resource count SLO check
- [x] Cost growth rate SLO check (structure)

#### V1

- [x] SLO types and schema
- [x] SLO manager with validation
- [x] Baseline-aware evaluation
- [x] Enforcement levels (Observe/Warn/Block/StrictBlock)
- [x] SLO report generation
- [x] Deployment blocking logic
- [ ] SLO inheritance (deferred to V2)
- [ ] Multi-SLO composition (deferred to V2)

#### Enterprise Burn Alerts (V2)

- [x] Burn rate linear regression
- [x] Time to breach prediction
- [x] CLI command: slo burn

---

### Mapping Engine

#### MVP (V1)

- [x] Resource to service mapping
- [x] Detect cross-service cost impacts
- [x] Infer downstream services (V1)
- [x] Implement cycle detection
- [x] Enforce max dependency depth (5)
- [x] Output Mermaid graph
- [x] Stable node IDs

#### V2

- [x] Graphviz export
  - [x] DOT format generation
  - [x] Configurable layout direction (LR, TB, RL, BT)
  - [x] Color schemes (cost-based, type-based, monochrome)
  - [x] Node styling by type (box, ellipse, folder)
  - [x] Edge styling by relationship (solid, dashed, dotted, bold)
  - [x] Module-based subgraph clustering
  - [x] Cost visualization on nodes
  - [x] Legend generation
  - [x] SVG/PNG export commands
- [x] JSON graph export
  - [x] Standard format (nodes/edges arrays)
  - [x] Adjacency list format
  - [x] Cytoscape.js format
  - [x] D3.js force-directed format
  - [x] Graph statistics
  - [x] Pretty printing option
  - [x] File export helper
- [x] CLI integration
  - [x] Multiple output formats (mermaid, graphviz, json, html)
  - [x] Format-specific options
  - [x] Output to file or stdout
  - [x] Verbose mode with statistics
- [ ] Cost propagation logic (V2)

---

### Grouping Engine

- [x] Group by module
  - [x] ModuleGroup with module_path, resources, monthly_cost, resource_count, cost_by_type
  - [x] Extract module path from resource address (root, root.vpc, root.vpc.subnets)
  - [x] Module depth tracking
  - [x] Parent/child module path navigation
  - [x] Aggregate module hierarchy (hierarchical rollup)
  - [x] Generate module tree for display
  - [x] Tests for module extraction and grouping
- [x] Group by service
  - [x] ServiceGroup with service_name, resources, monthly_cost, resource_count, cost_by_type, category
  - [x] ServiceCategory enum (10 categories: Compute, Storage, Database, Networking, Security, Analytics, ApplicationIntegration, Management, MachineLearning, Other)
  - [x] Extract service info from 50+ AWS resource types
  - [x] Group services by category
  - [x] Calculate cost by category
  - [x] Generate service cost summary report
  - [x] Tests for service extraction and categorization
- [x] Group by environment
  - [x] EnvironmentGroup with environment, resources, monthly_cost, resource_count, cost_by_type, cost_by_service
  - [x] Infer environment from tags (Environment, environment, Env, env)
  - [x] Infer environment from address patterns (prod, staging, dev, qa, uat, sandbox)
  - [x] Normalize environment names (prod->production, dev->development, stag->staging)
  - [x] Calculate environment cost ratios
  - [x] Detect environment anomalies (DevExceedsProd, StagingExceedsProd, UnknownEnvironmentHigh, ImbalancedCosts)
  - [x] Anomaly severity levels (High, Medium, Low)
  - [x] Generate environment cost report
  - [x] Tests for environment inference and anomaly detection

#### Attribution Pipeline

- [x] Extract tags
  - [x] AttributionPipeline with tag_mappings, default_environment, strict_matching
  - [x] Tag key mappings for environment, cost_center, owner, project, application
  - [x] Extract normalized tags from raw resource tags
  - [x] Custom tag mapping support
- [x] Normalize tag casing
  - [x] Lowercase keys, preserve values
  - [x] Canonical key variants (Environment -> environment, CostCenter -> cost_center)
- [x] Infer environment if missing
  - [x] Use environment inference from by_environment module
  - [x] Fallback to default_environment
- [x] Generate attribution report
  - [x] Attribution struct (resource_address, resource_type, environment, cost_center, owner, project, application, monthly_cost, tags)
  - [x] AttributionReport with cost breakdowns by environment/cost_center/owner/project/application
  - [x] Untagged cost tracking
  - [x] Tagging coverage calculation
  - [x] Top N cost centers/owners/projects
  - [x] Text report formatting
  - [x] CSV export
  - [x] JSON export
  - [x] Tests for tag extraction and attribution

#### Grouping Engine

- [x] GroupingEngine unified interface
  - [x] group_by_module(), group_by_service(), group_by_environment()
  - [x] generate_attribution_report()
  - [x] generate_comprehensive_report() (all dimensions)
  - [x] ComprehensiveReport with module/service/environment/attribution
  - [x] ComprehensiveReport text/JSON/CSV export
  - [x] GroupingOptions (min_cost_threshold, max_groups, sort_by, include_zero_cost)
  - [x] Tests for engine and comprehensive reporting

#### CLI Commands

- [x] `costpilot group module` - Group by Terraform module
  - [x] --tree flag for hierarchical tree view
  - [x] --min-cost threshold filter
  - [x] --max-groups limit
- [x] `costpilot group service` - Group by AWS service
  - [x] --by-category flag for category grouping
  - [x] --min-cost threshold filter
  - [x] --max-groups limit
- [x] `costpilot group environment` - Group by environment
  - [x] --detailed flag for detailed breakdown
  - [x] --detect-anomalies flag for anomaly detection
  - [x] --min-cost threshold filter
- [x] `costpilot group attribution` - Generate cost attribution report
  - [x] --format (text, json, csv)
  - [x] --output file path
  - [x] --top-n for top cost centers
- [x] `costpilot group all` - Comprehensive report across all dimensions
  - [x] --format (text, json)
  - [x] --output file path

---

### Trend Engine

#### V1

- [x] Write snapshot JSON
- [x] Validate snapshot schema
- [x] Generate SVG graph
- [x] Generate static HTML
- [x] Annotate regressions
- [x] Annotate SLO violations

#### Trend Snapshot Lifecycle

- [x] Snapshot rotation policy
- [x] Detect snapshot corruption
- [ ] Generate trend diff

---

## CLI

### Commands

- [x] `costpilot scan` (basic)
- [x] `costpilot scan --explain`
- [x] `costpilot diff`
- [x] `costpilot autofix snippet`
- [x] `costpilot autofix patch`
- [x] `costpilot init`
- [ ] `costpilot slo check`
- [x] `costpilot map --format=mermaid`
- [x] `costpilot map --format=graphviz`
- [x] `costpilot map --format=json`
- [x] `costpilot map --format=html`
- [x] `costpilot policy-dsl list`
- [x] `costpilot policy-dsl validate`
- [x] `costpilot policy-dsl test`
- [x] `costpilot policy-dsl stats`
- [x] `costpilot policy-dsl example`
- [x] `costpilot group module`
- [x] `costpilot group service`
- [x] `costpilot group environment`
- [x] `costpilot group attribution`
- [x] `costpilot group all`
- [x] `costpilot version`

### UX

- [x] Help text (enhanced with examples for main commands)
- [x] Examples (inline examples in help text)
- [x] Autocomplete (bash/zsh/fish)
- [ ] Version flag validation
- [ ] Machine-parseable output examples
- [ ] Debug mode support

---

## Zero-IAM Security & WASM

### Invariants

- [x] No network allowed
- [x] No AWS SDK allowed
- [x] WASM sandbox enforced
- [x] Redact secrets
- [x] Redact tokens
- [x] Enforce max file size (20MB)
- [x] Sandbox memory limit (256MB)
- [x] Sandbox timeout (2000ms)
- [x] Stable error signatures
- [x] Safe failure on corrupted input

### WASM Compile Pipeline

- [x] Compile core engines to WASM
  - [x] Cargo.toml configuration for wasm32-unknown-unknown target
  - [x] cdylib and rlib crate types
  - [x] wasm-release profile with size optimization (opt-level="z", lto=true, panic="abort")
  - [x] Feature flags for conditional compilation (prediction, detection, policy, mapping, grouping, slo)
  - [x] wasm-bindgen integration for JS bindings
  - [x] wee_alloc for smaller allocator
  - [x] Build script (scripts/build_wasm.sh) with optimization support
  - [x] Validation script (scripts/validate_wasm.sh)
- [x] Validate sandbox limits
  - [x] SandboxLimits struct (256 MB memory, 2000 ms timeout, 20 MB file size, 32 stack depth)
  - [x] EngineBudget constants for all 6 engines (prediction 300ms, detection 400ms, policy 200ms, mapping 500ms, grouping 400ms, slo 150ms)
  - [x] Input size validation
  - [x] JSON depth validation
  - [x] MemoryTracker for usage monitoring
  - [x] Tests for limit enforcement
- [x] WASM bytecode size limit
  - [x] 10 MB maximum size constraint
  - [x] Size validation in build script
  - [x] Size monitoring tests (wasm_size_tests.rs)
  - [x] Optimization strategies documented
  - [x] Compression potential analysis
  - [x] Feature flag size impact
  - [x] Regression detection (20% threshold)
- [x] WASM determinism test suite
  - [x] Prediction engine determinism tests
  - [x] Detection engine determinism tests
  - [x] Parser determinism tests
  - [x] HashMap iteration determinism (BTreeMap usage)
  - [x] No random value generation tests
  - [x] No system time usage tests
  - [x] No filesystem access tests
  - [x] Float determinism tests
  - [x] JSON serialization determinism
  - [x] Concurrent execution determinism
  - [x] Large plan determinism tests
  - [x] Output hash verification
- [x] WASM memory stress tests
  - [x] Memory limit validation tests
  - [x] Small plan memory tests (10 resources)
  - [x] Medium plan memory tests (1k resources)
  - [x] Large plan memory tests (10k resources)
  - [x] Memory pressure tests (repeated allocations)
  - [x] Nested JSON depth tests
  - [x] Repeated parsing stability tests
  - [x] Concurrent memory usage tests
  - [x] Allocation pattern tests
  - [x] String allocation stress tests
  - [x] HashMap allocation stress tests
  - [x] Vector growth tests
  - [x] Memory cleanup tests

#### WASM Runtime Infrastructure

- [x] Runtime wrapper module (src/wasm/runtime.rs)
  - [x] WASM initialization with panic hooks
  - [x] Sandbox limits configuration
  - [x] Engine budgets for all engines
  - [x] ValidationResult enum
  - [x] Input validation functions
  - [x] Memory tracking utilities
- [x] Module exports (src/wasm/mod.rs)
- [x] Library integration (src/lib.rs)
- [x] GitHub Actions CI workflow (.github/workflows/wasm.yml)
  - [x] WASM build job with caching
  - [x] Size validation (10 MB limit)
  - [x] Structure validation (wasm-validate)
  - [x] Security checks (no network imports)
  - [x] Test execution
  - [x] JS bindings generation
  - [x] Artifact upload
  - [x] Determinism test job
  - [x] Performance test job
- [x] Comprehensive documentation (WASM_BUILD.md)
  - [x] Architecture overview
  - [x] Build instructions
  - [x] Sandbox limits documentation
  - [x] Engine budgets table
  - [x] Determinism guarantees
  - [x] Runtime integration examples
  - [x] Testing strategies
  - [x] CI/CD integration
  - [x] Optimization strategies
  - [x] Security considerations
  - [x] Troubleshooting guide

---

## Performance Budgets

- [x] Max prediction latency: 300ms
- [x] Max mapping latency: 500ms
- [x] Max autofix generation: 400ms
- [x] Max total scan latency: 2000ms
- [x] WASM memory: 256MB
- [x] SLO engine max eval: 150ms

---

## Testing & Acceptance

### Testing Infrastructure (Completed)

- [x] Testing strategy document (TESTING_STRATEGY.md)
  - [x] 2,500 test target with pyramid distribution
  - [x] 100% code coverage goal
  - [x] Test distribution by category and engine
  - [x] Directory structure and organization
  - [x] Tools and frameworks selection
- [x] Test helpers module
  - [x] Fixtures for Terraform/CDK/CloudFormation/Policy/SLO
  - [x] Custom assertions for domain-specific validation
  - [x] Test data generators for property-based testing
- [x] Unit test templates
  - [x] Detection engine tests (350 tests planned)
  - [x] Prediction engine tests (400 tests planned)
  - [x] Explain engine tests (300 tests planned)
  - [x] Autofix engine tests (250 tests planned)
  - [x] Policy engine tests (300 tests planned)
  - [x] Mapping engine tests (200 tests planned)
  - [x] Grouping engine tests (250 tests planned)
  - [x] SLO engine tests (150 tests planned)
- [x] Integration test templates
  - [x] Full scan pipeline tests (30 tests planned)
  - [x] Policy + SLO enforcement tests (40 tests planned)
  - [x] Mapping + Grouping tests (30 tests planned)
  - [x] File I/O workflow tests (40 tests planned)
  - [x] WASM runtime tests (40 tests planned)
  - [x] Error recovery tests (20 tests planned)
- [x] E2E test templates
  - [x] CLI scan tests (10 tests planned)
  - [x] CLI autofix tests (8 tests planned)
  - [x] CLI map tests (8 tests planned)
  - [x] CLI group tests (10 tests planned)
  - [x] CLI policy tests (8 tests planned)
  - [x] CLI init tests (6 tests planned)
- [x] Code coverage configuration
  - [x] tarpaulin.toml with 90% threshold
  - [x] HTML, LCOV, XML output formats
  - [x] Exclusion patterns for non-source files
- [x] Performance benchmarks
  - [x] Criterion benchmarks for all engines
  - [x] Batch processing benchmarks
  - [x] Full pipeline benchmarks
- [x] CI/CD test automation
  - [x] Unit test job
  - [x] Integration test job
  - [x] E2E test job
  - [x] Coverage job with threshold enforcement
  - [x] Property-based test job
  - [x] Snapshot test job
  - [x] Fuzz test job (nightly)
  - [x] Mutation test job (weekly)
  - [x] Performance test job
  - [x] Lint and format job
- [x] Test dependencies in Cargo.toml
  - [x] Property-based: proptest, quickcheck
  - [x] Snapshot: insta
  - [x] Benchmarking: criterion
  - [x] E2E: assert_cmd, predicates, tempfile
  - [x] Utilities: rstest, test-case, fake, pretty_assertions

### Snapshot Tests (Infrastructure Ready)

- [ ] Terraform plan variants (50 tests planned)
- [ ] CDK diff variants (40 tests planned)
- [ ] CloudFormation variants (40 tests planned)
- [ ] Malformed plans (fuzz safety) (30 tests planned)
- [ ] Drift cases for autofix (40 tests planned)

### Fuzz Tests (Infrastructure Ready)

- [ ] Prediction engine (25 tests planned)
- [ ] Policy engine (25 tests planned)
- [ ] Mapping engine (25 tests planned)
- [ ] Terraform plan parsing resilience (25 tests planned)

### Golden File Tests (Infrastructure Ready)

- [ ] Predict output golden (50 tests planned)
- [ ] Explain output golden (40 tests planned)
- [ ] Mapping output golden (30 tests planned)
- [ ] Autofix output golden (30 tests planned)

### Baseline Tests (Infrastructure Ready)

- [ ] Baseline file missing behavior (12 tests planned)
- [ ] Baseline override handling (13 tests planned)
- [ ] Regression classifier uses baseline (12 tests planned)
- [ ] SLO uses baseline (13 tests planned)

### Acceptance Criteria

- [x] Identical inputs produce identical outputs (WASM determinism tests)
- [x] No network calls (Zero-network enforcement)
- [ ] Patch autofix requires simulation pass
- [ ] Cost diff never negative
- [x] Prediction intervals never inverted (Assertion helpers)
- [ ] Mapping graph valid JSON

### Error Model Enforcement

- [ ] Stable error IDs
- [ ] Error category mapping
- [ ] Remediation hint generator
- [ ] Machine message format tests

### Performance Regression Suite

- [x] Record perf baseline (Criterion benchmarks)
- [x] Diff against previous (GitHub Actions)
- [x] Enforce perf budgets (Budget infrastructure)

---

## Quality Contracts

### Execution Determinism Contract

- [x] Contract document (DETERMINISM_CONTRACT.md)
  - [x] 7 invariants defined (no entropy, no thread non-determinism, stable floats, stable JSON keys, markdown wrapping, LF newlines, deterministic errors)
  - [x] Required tests specified (cross-platform snapshot, float stability, parallel executor, JSON ordering, markdown consistency)
  - [x] Validation tools (bash determinism validator, float validator, JSON validator)
  - [x] CI enforcement (GitHub Actions matrix build with hash comparison)
  - [x] Breaking contract rules documented
  - [x] Success criteria (SHA-256 match, keys sorted, floats consistent, parallel identical)

### Heuristic Provenance Contract

- [x] Contract document (PROVENANCE_CONTRACT.md)
  - [x] Data structures (HeuristicProvenance, ConfidenceSource, FallbackReason)
  - [x] 3 invariants (explain must reference provenance, hash deterministic, missing provenance is error)
  - [x] Explain engine integration (provenance per ReasoningStep)
  - [x] JSON output schema with provenance
  - [x] Validation tests (all predictions have provenance, hash determinism, cold-start provenance, schema validation)
  - [x] CLI integration (--show-provenance flag, validate-provenance command)

### Grammar and Style Contract

- [x] Contract document (GRAMMAR_CONTRACT.md)
  - [x] 6 core principles (no hedging, no undefined terms, no randomized wording, stable templates, severity consistency, currency formatting)
  - [x] Style rules (capitalization, punctuation, abbreviations)
  - [x] Message templates (cost analysis, policy violations, recommendations)
  - [x] Formatting rules (numbers, percentages, time durations)
  - [x] Markdown output rules (headers, lists, code blocks, tables)
  - [x] PR comment format template and example
  - [x] Validation tests (no hedging language, costs have currency symbols, severity consistency)

### Canonical Layout Contract

- [x] Contract document (CANONICAL_LAYOUT_CONTRACT.md)
  - [x] JSON canonical layout (2-space indent, LF newlines, BTreeMap for stable keys, no trailing comma)
  - [x] Markdown canonical layout (80 char line width, ATX headers, aligned tables)
  - [x] Mermaid canonical layout (deterministic node IDs, sorted edges)
  - [x] SVG canonical layout (fixed dimensions, 2 decimal precision, alphabetical attributes)
  - [x] CLI output layout (80 char max, ANSI colors if TTY)
  - [x] File naming conventions (costpilot-{type}-{identifier}.{ext})
  - [x] Validation tests (JSON deterministic, Markdown line width, Mermaid edge ordering)

### Behavior Freeze Contract

- [x] Contract document (BEHAVIOR_FREEZE_CONTRACT.md)
  - [x] Semantic versioning rules (MAJOR.MINOR.PATCH)
  - [x] Frozen behaviors (regression classifier, prediction semantics, mapping schema, explain schemas, heuristics format, policy evaluation)
  - [x] CLI interface stability (command structure, exit codes)
  - [x] Output format stability (JSON schema versioning, Markdown format)
  - [x] Breaking change process (deprecation warnings, migration guides)
  - [x] CHANGELOG.md format template
  - [x] Validation tests (regression classifier stability, JSON schema version, CLI exit codes)

### Error Signatures Contract

- [x] Contract document (ERROR_SIGNATURES_CONTRACT.md)
  - [x] Error signature structure (code, category, message, context, hint, hash)
  - [x] Error codes (E001-E599) with categories (Parse, Validation, Runtime, IO, Configuration, Internal)
  - [x] Error formatting (terminal output with emoji, JSON schema)
  - [x] Builder pattern for error construction
  - [x] CLI exit codes (0=success, 1=unknown, 2=policy, 10-15=categories)
  - [x] Actionable hints guidelines
  - [x] Error logging (structured logging, metrics)
  - [x] Validation tests (hash determinism, all codes have categories, JSON serialization)

### Regression Justification Contract

- [x] Contract document (REGRESSION_JUSTIFICATION_CONTRACT.md)
  - [x] 6 mandatory elements (type, driver, delta, confidence, dependencies, root cause)
  - [x] Data structures (RegressionJustification, RegressionType, RegressionDriver, CostDelta, DependencyContext, RootCause)
  - [x] Recommendation structure (action, impact, confidence, effort)
  - [x] PR comment format template
  - [x] Validation rules (confidence range, delta consistency, root cause explanation)
  - [x] Completeness checks
  - [x] Tests (justification validation, PR comment format, mandatory fields)

### PR Comment Quality Contract

- [x] Contract document (PR_COMMENT_QUALITY_CONTRACT.md)
  - [x] Marketing-quality format (screenshot-worthy)
  - [x] Template structure (💰 Analysis, 📊 Summary, 🔍 Findings, 💡 Recommendations, 📈 Confidence)
  - [x] Line limit enforcement (max 15 lines before <details>)
  - [x] Copy-paste safety (no special characters, no trailing whitespace, no tabs)
  - [x] Emoji consistency (standard emoji set)
  - [x] Detailed breakdown format (tables for resources)
  - [x] Real-world examples (cost increase, decrease, complex change)
  - [x] Validation tests (format valid, copy-paste safe, currency formatting)

### Self-Consistency Tests Contract

- [x] Contract document (SELF_CONSISTENCY_TESTS_CONTRACT.md)
  - [x] 10 consistency checks (Detection↔Prediction, Prediction↔Explain, Mapping↔Grouping, Policy↔Severity, Heuristic↔Confidence, Regression↔Delta, Float Math, JSON Schema, Error Codes, CLI Exit Codes)
  - [x] Test implementations for each check
  - [x] Meta-test runner (runs all consistency tests)
  - [x] CI integration (self-consistency job)
  - [x] Fuzzing for consistency (property-based tests)
  - [x] Benefits documented (bug detection, code quality, user trust)

---

## Compatibility Enforcement

- [ ] API contract golden file
- [ ] Heuristics version check
- [ ] Semver enforcement tests
- [ ] Canonical output schema validation

---

## Config Validation Engine

- [x] Validate costpilot.yaml
  - [x] Schema validation with version/region/scan/policies/output/heuristics/SLO/integrations
  - [x] Semantic validation (semver, AWS regions, output formats, duration formats, Slack webhooks)
  - [x] Error codes (E100-E105) with remediation hints
  - [x] Warnings for missing optional fields
- [x] Validate policy files
  - [x] Policy structure validation (metadata, rules, exemptions)
  - [x] Rule validation (name, conditions, actions, severity, enabled status)
  - [x] Condition validation (operators, values, regex patterns)
  - [x] Exemption validation (resource patterns, reasons, expiry dates)
  - [x] Error codes (E200-E207) with helpful hints
- [x] Validate baselines file
  - [x] BaselinesConfig validation (version, global, modules, services)
  - [x] Baseline validation (expected_monthly_cost, last_updated, justification, owner)
  - [x] Cost validation (non-negative, non-zero warnings)
  - [x] Timestamp validation (RFC3339 format)
  - [x] Staleness detection (90-day threshold warnings)
  - [x] Error codes (E300-E303) with remediation hints
- [x] Validate SLO files
  - [x] SLO structure validation (id, name, description, slo_type, target, threshold, enforcement)
  - [x] Threshold validation (max_value, min_value, warning_threshold_percent)
  - [x] Enforcement level validation with blocking warnings
  - [x] Target validation (non-empty entity names)
  - [x] Error codes (E400-E406) with helpful hints
- [x] CLI command: `costpilot validate`
  - [x] Single file validation
  - [x] Batch validation with --fail-fast option
  - [x] JSON and text output formats
  - [x] Summary statistics for batch validation
  - [x] Exit code 2 for validation failures
- [x] Validation module structure
  - [x] ValidationReport with errors and warnings
  - [x] ValidationError with field/line/column/hint/error_code
  - [x] ValidationWarning with field/suggestion/warning_code
  - [x] Colored terminal output with emoji
  - [x] Automatic file type detection from name/extension

---

## Pricing & Licensing

### Solo

- [ ] Snippet autofix
- [ ] Limited explain
- [ ] Unlimited repos

### Pro

- [ ] Patch autofix
- [ ] Drift-safe autofix
- [ ] Mapping graph export
- [ ] Policy as code (lite)

### Enterprise

- [x] SLO burn alerts
- [ ] Full PAC engine
- [ ] Exemption workflow
- [ ] Team cost attribution
- [ ] Audit logs and tamper-proofing
- [ ] Software escrow

---

## Governance & Lifecycle

- [ ] Policy version increment on change
- [ ] Approval reference required if flagged
- [ ] Expired exemptions block CI
- [ ] Drift detection checksum (SHA256)
- [ ] Critical drift blocks execution

---

## Go-To-Market

### Assets

- [ ] Demo PR repo
- [ ] Demo GIF (60s)
- [ ] TCO calculator
- [ ] Launch day blog
- [ ] Product Hunt launch
- [ ] Hacker News Show launch

### Marketplace Submission

- [ ] Metadata JSON
- [ ] Pricing mapping
- [ ] README action
- [ ] Icon assets

### GitHub Action Packaging

- [ ] Generate action.yml
- [ ] Embed binary or fetch script
- [ ] Test in ephemeral repo
- [ ] Verify action permissions
- [ ] Publish versioned releases

### SEO

- [ ] Docs indexable
- [ ] Landing page keywords
- [ ] Schema metadata
- [ ] Example query: "terraform cost diff"
- [ ] Example query: "aws finops linter"

---

## Onboarding

- [ ] `costpilot init` generates CI template
- [ ] Sample policies and baselines
- [ ] Mermaid mapping sample
- [ ] CLI quickstart guide

---

## Release Pipeline

- [ ] Tag release
- [ ] Generate changelog
- [ ] Update spec version
- [ ] Rebuild WASM
- [ ] Publish GitHub release

---

### 🎯 Legend

✅ **Completed** | 🚧 **In Progress** | ⏳ **Pending** | ⚠️ **Blocked**

---

**Last Updated:** 2025-12-06  
**Next Milestone:** MVP (Trust Triangle) - Days 0-14

</div>
