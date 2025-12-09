# CostPilot Mermaid Diagram Examples

## Overview

This document provides Mermaid diagram examples for visualizing CostPilot workflows, cost mappings, policy flows, and architecture.

## Workflow Diagrams

### Basic Cost Analysis Workflow

```mermaid
flowchart LR
    A[Terraform Plan] --> B[Parse JSON]
    B --> C[Extract Resources]
    C --> D[Calculate Costs]
    D --> E[Compare Baseline]
    E --> F[Check Policies]
    F --> G{Violations?}
    G -->|Yes| H[Generate Report]
    G -->|No| I[Approve]
    H --> J[Post PR Comment]
    J --> K{Critical?}
    K -->|Yes| L[Block Merge]
    K -->|No| M[Allow with Warning]
```

### Full CI/CD Integration

```mermaid
flowchart TD
    Start[Developer Creates PR] --> TF[Terraform Plan]
    TF --> CP[CostPilot Analyze]
    
    CP --> Parse[Parse Plan JSON]
    Parse --> Cost[Calculate Costs]
    Cost --> Baseline[Compare Baseline]
    Baseline --> Policy[Evaluate Policies]
    Policy --> Drift[Detect Drift]
    Drift --> Predict[AI Predictions]
    
    Predict --> Report[Generate Report]
    Report --> Comment[Post PR Comment]
    
    Comment --> Check{Pass All Checks?}
    Check -->|Yes| Approve[✓ Approve Merge]
    Check -->|Policy Violation| Review[⚠ Require Approval]
    Check -->|Critical Drift| Block[❌ Block Merge]
    
    Review --> Manual[Manual Review]
    Manual --> Decision{Exemption Granted?}
    Decision -->|Yes| Approve
    Decision -->|No| Block
    
    Block --> Fix[Developer Fixes]
    Fix --> Start
```

### Policy Evaluation Flow

```mermaid
flowchart TD
    Start[Load Policies] --> Parse[Parse Policy DSL]
    Parse --> Resources[Get Changed Resources]
    
    Resources --> Loop{For Each Resource}
    Loop --> Eval[Evaluate Policy Rules]
    
    Eval --> Match{Rule Matches?}
    Match -->|Yes| Action{Check Action}
    Match -->|No| Loop
    
    Action -->|block| Block[Add Blocking Violation]
    Action -->|warn| Warn[Add Warning]
    Action -->|require_approval| Approval[Flag for Approval]
    
    Block --> Exemption{Exemption Exists?}
    Exemption -->|Yes| Expired{Expired?}
    Exemption -->|No| Record[Record Violation]
    
    Expired -->|Yes| Record
    Expired -->|No| Skip[Skip Violation]
    
    Warn --> Record
    Approval --> Record
    Skip --> Loop
    Record --> Loop
    
    Loop -->|More Resources| Eval
    Loop -->|Done| Summary[Generate Summary]
    Summary --> End[Return Results]
```

## Cost Mapping Diagrams

### Resource to Cost Mapping

```mermaid
graph LR
    subgraph Terraform Resources
        R1[aws_instance.web]
        R2[aws_rds_instance.db]
        R3[aws_s3_bucket.assets]
        R4[aws_nat_gateway.main]
    end
    
    subgraph Cost Components
        C1[Compute: $50/mo]
        C2[Database: $200/mo]
        C3[Storage: $10/mo]
        C4[Network: $45/mo]
    end
    
    subgraph Total Cost
        T[Monthly Total: $305]
    end
    
    R1 --> C1
    R2 --> C2
    R3 --> C3
    R4 --> C4
    
    C1 --> T
    C2 --> T
    C3 --> T
    C4 --> T
    
    style T fill:#e1f5e1
    style R1 fill:#fff3cd
    style R2 fill:#fff3cd
    style R3 fill:#fff3cd
    style R4 fill:#fff3cd
```

### Module Cost Breakdown

```mermaid
pie title Monthly Cost Distribution
    "Compute (EC2)" : 30
    "Database (RDS)" : 40
    "Storage (S3)" : 10
    "Network (NAT/VPC)" : 15
    "Other Services" : 5
```

### Cost Trend Over Time

```mermaid
graph LR
    M1[Month 1<br/>$250] --> M2[Month 2<br/>$280]
    M2 --> M3[Month 3<br/>$310]
    M3 --> M4[Month 4<br/>$295]
    M4 --> M5[Month 5<br/>$420]
    
    M5 -.->|Predicted| M6[Month 6<br/>$450]
    M6 -.-> M7[Month 7<br/>$480]
    
    style M5 fill:#ffcccc
    style M6 fill:#cce5ff
    style M7 fill:#cce5ff
```

## Architecture Diagrams

### CostPilot System Architecture

```mermaid
graph TB
    subgraph User Interface
        CLI[CLI Tool]
        GHA[GitHub Action]
        API[REST API]
    end
    
    subgraph Core Engine
        Parser[Plan Parser]
        Analyzer[Cost Analyzer]
        PolicyEngine[Policy Engine]
        DriftDetector[Drift Detector]
        Predictor[AI Predictor]
    end
    
    subgraph Data Layer
        Pricing[Pricing Data]
        Baseline[Baseline Store]
        History[Historical Data]
        Cache[Cost Cache]
    end
    
    subgraph Output
        Report[Markdown Report]
        JSON[JSON Output]
        PRComment[PR Comment]
        Metrics[Metrics/Logs]
    end
    
    CLI --> Parser
    GHA --> Parser
    API --> Parser
    
    Parser --> Analyzer
    Analyzer --> PolicyEngine
    PolicyEngine --> DriftDetector
    DriftDetector --> Predictor
    
    Analyzer --> Pricing
    PolicyEngine --> Baseline
    Predictor --> History
    Analyzer --> Cache
    
    Predictor --> Report
    Predictor --> JSON
    Predictor --> PRComment
    Predictor --> Metrics
```

### Drift Detection Flow

```mermaid sequenceDiagram
    participant TF as Terraform
    participant CP as CostPilot
    participant Baseline as Baseline Store
    participant Checksum as SHA256 Engine
    
    TF->>CP: Submit plan.json
    CP->>Checksum: Calculate current checksum
    Checksum-->>CP: SHA256 hash
    
    CP->>Baseline: Fetch expected checksum
    Baseline-->>CP: Previous SHA256 hash
    
    CP->>CP: Compare checksums
    
    alt Checksums Match
        CP-->>TF: ✓ No drift detected
    else Checksums Differ
        CP->>CP: Identify drifted attributes
        CP->>CP: Assess criticality
        
        alt Critical Drift
            CP-->>TF: ❌ Block execution
        else Non-Critical Drift
            CP-->>TF: ⚠ Warning issued
        end
    end
```

### Policy Enforcement Sequence

```mermaid sequenceDiagram
    participant Dev as Developer
    participant PR as Pull Request
    participant CI as CI/CD
    participant CP as CostPilot
    participant Policy as Policy Engine
    participant Approver as Tech Lead
    
    Dev->>PR: Create PR with changes
    PR->>CI: Trigger workflow
    CI->>CP: Run cost analysis
    
    CP->>Policy: Load policies
    Policy->>Policy: Evaluate rules
    
    alt No Violations
        Policy-->>CP: ✓ Pass
        CP-->>CI: Approve
        CI-->>PR: ✓ Checks pass
        PR-->>Dev: Can merge
    else Warning Level
        Policy-->>CP: ⚠ Warnings
        CP-->>CI: Pass with warnings
        CI-->>PR: ⚠ Review recommended
        PR-->>Dev: Can merge with caution
    else Requires Approval
        Policy-->>CP: 🔔 Approval needed
        CP-->>CI: Request approval
        CI-->>PR: 🔔 Awaiting approval
        PR-->>Approver: Notify for review
        Approver->>PR: Review and approve
        PR-->>Dev: Can merge after approval
    else Blocking Violation
        Policy-->>CP: ❌ Block
        CP-->>CI: Fail check
        CI-->>PR: ❌ Checks failed
        PR-->>Dev: Cannot merge - fix required
    end
```

## Data Flow Diagrams

### Cost Calculation Pipeline

```mermaid
flowchart LR
    subgraph Input
        Plan[plan.json]
        Pricing[AWS Pricing API]
        Baseline[baseline.json]
    end
    
    subgraph Processing
        Parse[Parse Resources]
        Map[Map to Cost Units]
        Calculate[Calculate Costs]
        Aggregate[Aggregate Totals]
    end
    
    subgraph Analysis
        Compare[Compare vs Baseline]
        Trends[Analyze Trends]
        Predict[AI Predictions]
    end
    
    subgraph Output
        Report[Cost Report]
        Alerts[Alerts/Notifications]
        Metrics[Usage Metrics]
    end
    
    Plan --> Parse
    Pricing --> Map
    Parse --> Map
    Map --> Calculate
    Calculate --> Aggregate
    
    Aggregate --> Compare
    Baseline --> Compare
    Compare --> Trends
    Trends --> Predict
    
    Predict --> Report
    Predict --> Alerts
    Predict --> Metrics
```

### Baseline Update Process

```mermaid
stateDiagram-v2
    [*] --> LoadBaseline
    LoadBaseline --> AnalyzeCurrent
    AnalyzeCurrent --> CompareCosts
    
    CompareCosts --> NoChange: Costs unchanged
    CompareCosts --> MinorChange: <10% change
    CompareCosts --> MajorChange: ≥10% change
    
    NoChange --> [*]
    MinorChange --> AutoUpdate: Auto-update enabled
    MinorChange --> ManualReview: Auto-update disabled
    MajorChange --> ManualReview
    
    AutoUpdate --> SaveBaseline
    ManualReview --> ApprovalRequired
    ApprovalRequired --> Approved: Approved
    ApprovalRequired --> Rejected: Rejected
    
    Approved --> SaveBaseline
    Rejected --> [*]
    SaveBaseline --> [*]
```

## Use Case Scenarios

### Scenario 1: Cost Regression Detected

```mermaid
journey
    title Developer Experience: Cost Regression
    section Create PR
      Write Terraform code: 5: Developer
      Create pull request: 5: Developer
    section CI Pipeline
      Terraform plan: 3: CI
      CostPilot analysis: 3: CI
      Detect regression: 1: CostPilot
    section Review
      Read PR comment: 2: Developer
      Review cost increase: 2: Developer
      Check recommendations: 3: Developer
    section Resolution
      Optimize resources: 4: Developer
      Rerun analysis: 5: Developer
      Merge approved: 5: Developer
```

### Scenario 2: Policy Violation

```mermaid
journey
    title Engineer Experience: Policy Violation
    section Development
      Add NAT gateway: 5: Engineer
      Submit PR: 5: Engineer
    section Validation
      Policy check runs: 3: CI
      Violation detected: 1: CostPilot
      PR comment posted: 2: CostPilot
    section Escalation
      Review violation: 2: Engineer
      Request exemption: 3: Engineer
      Tech lead notified: 3: System
    section Approval
      Tech lead reviews: 4: Tech Lead
      Exemption granted: 5: Tech Lead
      Merge approved: 5: Engineer
```

### Scenario 3: Drift Detection

```mermaid
journey
    title SRE Experience: Drift Detection
    section Normal Operation
      Deploy via Terraform: 5: SRE
      Baseline saved: 5: CostPilot
    section Manual Change
      Console modification: 1: Unknown
      Next PR created: 3: SRE
    section Detection
      Drift checksum fails: 1: CostPilot
      Critical drift flagged: 1: CostPilot
      PR blocked: 1: CI
    section Investigation
      Review drift report: 2: SRE
      Identify manual change: 2: SRE
      Update Terraform: 4: SRE
    section Resolution
      Drift resolved: 5: SRE
      Baseline updated: 5: CostPilot
      Deploy succeeds: 5: SRE
```

## Integration Diagrams

### GitHub Actions Integration

```mermaid
graph TD
    Event[PR Event] --> Trigger[Trigger Workflow]
    Trigger --> Checkout[Checkout Code]
    Checkout --> TFSetup[Setup Terraform]
    TFSetup --> TFInit[Terraform Init]
    TFInit --> TFPlan[Terraform Plan]
    TFPlan --> TFShow[Convert to JSON]
    
    TFShow --> CPAction[CostPilot Action]
    CPAction --> Analysis[Run Analysis]
    
    Analysis --> Estimate[Cost Estimation]
    Analysis --> Policy[Policy Check]
    Analysis --> Drift[Drift Detection]
    Analysis --> Predict[AI Prediction]
    
    Estimate --> Report[Generate Report]
    Policy --> Report
    Drift --> Report
    Predict --> Report
    
    Report --> Comment[Post PR Comment]
    Report --> Status{Check Status}
    
    Status -->|Pass| Success[✓ Success]
    Status -->|Warning| Neutral[⚠ Neutral]
    Status -->|Fail| Failure[❌ Failure]
    
    Success --> Artifact[Upload Artifacts]
    Neutral --> Artifact
    Failure --> Artifact
```

### Multi-Environment Workflow

```mermaid
graph LR
    subgraph Development
        DevPR[Dev PR] --> DevAnalysis[Cost Analysis]
        DevAnalysis --> DevBaseline[Dev Baseline]
    end
    
    subgraph Staging
        StgPR[Staging PR] --> StgAnalysis[Cost Analysis]
        StgAnalysis --> StgBaseline[Staging Baseline]
    end
    
    subgraph Production
        ProdPR[Production PR] --> ProdAnalysis[Cost Analysis]
        ProdAnalysis --> ProdBaseline[Prod Baseline]
        ProdBaseline --> StrictPolicy[Strict Policies]
        StrictPolicy --> Approval[Require Approval]
    end
    
    DevBaseline -.->|Promote| StgBaseline
    StgBaseline -.->|Promote| ProdBaseline
```

## Visualization Examples

### Cost Impact Heatmap

```mermaid
graph TD
    subgraph High Impact - High Cost
        H1[RDS Multi-AZ<br/>+$400/mo]
        H2[NAT Gateway x3<br/>+$350/mo]
    end
    
    subgraph High Impact - Low Cost
        M1[S3 Lifecycle<br/>-$50/mo]
        M2[Lambda Provisioned<br/>-$80/mo]
    end
    
    subgraph Low Impact - High Cost
        L1[EC2 Reserved<br/>+$200/mo]
    end
    
    subgraph Low Impact - Low Cost
        S1[CloudWatch Logs<br/>+$15/mo]
        S2[Route53 Zones<br/>+$5/mo]
    end
    
    style H1 fill:#ff6b6b
    style H2 fill:#ff6b6b
    style M1 fill:#ffd93d
    style M2 fill:#ffd93d
    style L1 fill:#a8dadc
    style S1 fill:#e1f5e1
    style S2 fill:#e1f5e1
```

### Decision Tree for Policy Actions

```mermaid
graph TD
    Start[Policy Evaluation] --> CostCheck{Cost Increase?}
    
    CostCheck -->|<10%| Low[Low Impact]
    CostCheck -->|10-30%| Med[Medium Impact]
    CostCheck -->|>30%| High[High Impact]
    
    Low --> AutoApprove[✓ Auto-Approve]
    
    Med --> PolicyCheck{Policy Violation?}
    PolicyCheck -->|No| AutoApprove
    PolicyCheck -->|Yes| Warn[⚠ Warning]
    
    High --> RequireApproval[🔔 Require Approval]
    
    RequireApproval --> Review[Manual Review]
    Review --> Decision{Approved?}
    Decision -->|Yes| Exemption[Grant Exemption]
    Decision -->|No| Block[❌ Block]
    
    Exemption --> TimeLimit{Time-Bound?}
    TimeLimit -->|Yes| SetExpiry[Set Expiration]
    TimeLimit -->|No| Permanent[Permanent Exemption]
    
    SetExpiry --> Approve[✓ Approve]
    Permanent --> Approve
```

## Summary

These Mermaid diagrams can be embedded in documentation, pull requests, and dashboards to visualize:

- **Workflows**: How CostPilot integrates into CI/CD
- **Cost Mappings**: How resources map to costs
- **Policy Flows**: How policies are evaluated and enforced
- **Architecture**: System components and interactions
- **User Journeys**: Developer/SRE experience

### Usage in Markdown

```markdown
# Example: Embedding in Documentation

## Cost Analysis Workflow

\`\`\`mermaid
flowchart LR
    A[Terraform Plan] --> B[CostPilot]
    B --> C[Report]
\`\`\`
```

### Rendering

- **GitHub**: Renders Mermaid natively in READMEs and PRs
- **GitLab**: Supports Mermaid in markdown
- **VS Code**: Use Mermaid Preview extension
- **Documentation Sites**: MkDocs, Docusaurus support Mermaid

---

**Last Updated:** December 2025
**Version:** 1.0.0
