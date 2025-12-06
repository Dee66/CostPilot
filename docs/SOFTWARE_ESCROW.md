# Software Escrow Documentation

## Overview

CostPilot's Software Escrow system provides enterprise-grade business continuity for customers. In the event of vendor insolvency, acquisition, or other trigger conditions, customers can access the complete source code, build instructions, and dependencies needed to maintain and deploy CostPilot independently.

## Why Software Escrow?

### Business Continuity
Protect your investment in CostPilot. If the vendor can no longer provide support or updates, you have everything needed to continue operations.

### Risk Mitigation
Reduce vendor lock-in and ensure long-term access to mission-critical infrastructure cost management capabilities.

### Compliance Requirements
Many enterprises require software escrow agreements as part of procurement processes, especially for critical infrastructure tools.

### Trust & Transparency
Demonstrates vendor commitment to customer success beyond the typical software-as-a-service model.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Escrow System Architecture                    │
│                                                                   │
│  ┌────────────┐    ┌─────────────┐    ┌──────────────────┐     │
│  │  Package   │───▶│  Release    │───▶│  Escrow Agent    │     │
│  │  Builder   │    │  Automation │    │  (Iron Mountain) │     │
│  └────────────┘    └─────────────┘    └──────────────────┘     │
│        │                   │                      │              │
│        │                   │                      │              │
│        ▼                   ▼                      ▼              │
│  ┌────────────┐    ┌─────────────┐    ┌──────────────────┐     │
│  │ Source     │    │ Build       │    │ Verification     │     │
│  │ Files      │    │ Instructions│    │ & Testing        │     │
│  │ + Checksums│    │ + Deps      │    │                  │     │
│  └────────────┘    └─────────────┘    └──────────────────┘     │
│                                                                   │
│                          Trigger Event                           │
│                               │                                   │
│                               ▼                                   │
│                    ┌──────────────────┐                          │
│                    │    Recovery      │                          │
│                    │   Orchestrator   │                          │
│                    └──────────────────┘                          │
│                               │                                   │
│                               ▼                                   │
│                    ┌──────────────────┐                          │
│                    │  Rebuilt System  │                          │
│                    │   (Operational)  │                          │
│                    └──────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Escrow Package

A complete, self-contained snapshot of CostPilot at a specific version.

**Contents:**
- **Source Code**: All Rust, TypeScript, and configuration files
- **Build Artifacts**: Pre-compiled binaries and WASM modules
- **Dependencies**: Complete dependency manifest with versions and checksums
- **Build Instructions**: Step-by-step compilation and deployment guide
- **Tests**: Full test suite for verification
- **Documentation**: Technical documentation and API references
- **License**: Complete license information and open source components

**Package Structure:**
```
escrow-package/
├── escrow-package.json      # Package manifest
├── ESCROW-README.md         # Quick start guide
├── source/                  # Source code
│   ├── src/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── ...
├── artifacts/               # Build artifacts
│   ├── costpilot            # Binary
│   └── costpilot.wasm       # WASM module
├── dependencies/            # Dependency cache
│   └── cargo-cache/
└── playbook.md             # Recovery playbook
```

### 2. Release Automation

Automatically creates escrow deposits on each release.

**Triggers:**
- Git tags matching `v*.*.*` pattern
- Version changes in `Cargo.toml`
- Manual invocation

**Process:**
1. Detect release event
2. Collect all source files with checksums
3. Build artifacts (if configured)
4. Generate dependencies manifest
5. Create build instructions
6. Verify package completeness
7. Deposit to escrow location
8. Upload to escrow agent (if configured)
9. Generate deposit receipt

### 3. Verification System

Ensures package integrity and completeness.

**Checks:**
- ✅ All required files present (Cargo.toml, LICENSE, README.md)
- ✅ All checksums valid
- ✅ Dependencies complete
- ✅ Build instructions valid
- ✅ No critical vulnerabilities
- ✅ Package checksum matches

**Verification Report:**
```
🔒 Escrow Package Verification Report
=====================================

✅ Status: VERIFIED

Information:
  ℹ️  Package version: 1.0.0
  ℹ️  Source files: 247
  ℹ️  Build artifacts: 2
  ℹ️  Dependencies: 89
```

### 4. Recovery Orchestrator

Guides customers through building from escrow package.

**Recovery Steps:**
1. **Verify Package Integrity** - Validate checksums and completeness
2. **Check Prerequisites** - Rust, Cargo, Node.js versions
3. **Extract Source Files** - Unpack to working directory
4. **Install Dependencies** - Run `cargo fetch` and `npm install`
5. **Build from Source** - Execute build steps sequentially
6. **Run Tests** - Verify build correctness
7. **Generate Deployment Package** - Create production-ready artifacts

**Recovery Report:**
```
🔄 Escrow Recovery Report
=========================

✅ Status: RECOVERY SUCCESSFUL

Total Duration: 847 seconds

Completed Steps:
----------------
1. Verify package integrity (2 seconds)
   Package integrity verified successfully

2. Check prerequisites (3 seconds)
   ✅ Rust: rustc 1.75.0
   ✅ Cargo: cargo 1.75.0
   ✅ Node.js: v18.17.0

3. Extract source files (5 seconds)
   Extracted 247 source files to /tmp/recovery/source

4. Install dependencies (120 seconds)
   ✅ Installed 89 Cargo dependencies
   ✅ Installed 23 NPM dependencies

5. Build from source (600 seconds)
   === Step 1: Install Rust toolchain ===
   ✅ Step completed successfully
   
   === Step 2: Build release binary ===
   ✅ Step completed successfully
   
   === Step 3: Run tests ===
   ✅ Step completed successfully
   
   === Step 4: Build WASM target ===
   ✅ Step completed successfully

6. Run tests (100 seconds)
   ✅ Tests passed

7. Generate deployment package (17 seconds)
   ✅ Packaged: costpilot
   ✅ Packaged: costpilot.wasm
   
   📦 Deployment package ready at: /tmp/recovery/deployment
```

## Escrow Triggers

### Standard Trigger Conditions

**Bankruptcy or Insolvency**
- Vendor files for bankruptcy protection
- Receiver or administrator appointed
- Voluntary liquidation

**Acquisition**
- Company acquired by competitor
- Change of control to restricted entity
- Material breach of acquisition terms

**Failure to Provide Support**
- Critical bugs unresolved for 90 days
- No security updates for 180 days
- Service level agreements breached

**Material Breach**
- Vendor materially breaches agreement
- Failure to maintain escrow deposits
- Cessation of product development

**Voluntary Release**
- Open source transition
- Product end-of-life
- Vendor discretion

### Custom Trigger Conditions

Enterprises can negotiate custom trigger conditions:
- M&A with specific competitors
- Changes in ownership structure
- Financial performance metrics
- Compliance failures

## CLI Commands

### Create Escrow Package

```bash
# Create package for current version
costpilot escrow create --version 1.0.0

# Include build artifacts
costpilot escrow create --version 1.0.0 --include-artifacts

# Specify output directory
costpilot escrow create --version 1.0.0 --output ./escrow-packages
```

**Output:**
```
✅ Escrow package created successfully

📦 Escrow Deposit Receipt
========================

Receipt ID: 550e8400-e29b-41d4-a716-446655440000
Package ID: 7c9e6679-7425-40de-944b-e07fc1f90ae7
Version: 1.0.0
Deposit Date: 1701388800
Location: ./escrow-packages/1.0.0
Escrow Agent: Iron Mountain
Status: Completed

This receipt confirms the deposit of CostPilot v1.0.0 to software escrow.
The package has been securely stored and is available for release under
the terms of the escrow agreement.
```

### Verify Package

```bash
# Verify package integrity
costpilot escrow verify ./escrow-packages/1.0.0
```

**Output:**
```
🔒 Escrow Package Verification Report
=====================================

✅ Status: VERIFIED

Information:
  ℹ️  Package version: 1.0.0
  ℹ️  Source files: 247
  ℹ️  Build artifacts: 2
  ℹ️  Dependencies: 89
```

### Generate Recovery Playbook

```bash
# Generate playbook
costpilot escrow playbook ./escrow-packages/1.0.0

# Save to file
costpilot escrow playbook ./escrow-packages/1.0.0 --output recovery.md
```

**Output:**
```
📖 Recovery playbook written to: recovery.md
```

### Recover from Escrow

```bash
# Run complete recovery process
costpilot escrow recover \
  --package ./escrow-packages/1.0.0 \
  --working-dir /tmp/recovery
```

**Output:**
Recovery report showing all steps and their status.

### Configure Escrow Settings

```bash
# Set vendor information
costpilot escrow configure \
  --vendor "Acme Corp" \
  --contact support@acme.com \
  --support https://acme.com/support
```

### List Packages

```bash
# List all escrow packages
costpilot escrow list

# List from specific directory
costpilot escrow list --dir /path/to/escrow-packages
```

**Output:**
```
📦 Escrow Packages
==================

1. Version: 1.0.0
   Date: 1701388800
   Commit: abc123def456
   Location: ./escrow-packages/1.0.0

2. Version: 1.1.0
   Date: 1704067200
   Commit: 789ghi012jkl
   Location: ./escrow-packages/1.1.0
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Escrow Deposit

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  escrow:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install CostPilot
        run: cargo install costpilot
      
      - name: Create Escrow Package
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          costpilot escrow create \
            --version $VERSION \
            --include-artifacts \
            --output ./escrow-packages
      
      - name: Upload to Escrow Agent
        env:
          ESCROW_API_KEY: ${{ secrets.ESCROW_API_KEY }}
        run: |
          # Upload via escrow agent API
          curl -X POST https://escrow-api.example.com/deposit \
            -H "Authorization: Bearer $ESCROW_API_KEY" \
            -F "package=@escrow-packages/$VERSION.tar.gz"
```

### GitLab CI

```yaml
escrow:
  stage: release
  only:
    - tags
  script:
    - cargo install costpilot
    - VERSION=${CI_COMMIT_TAG#v}
    - costpilot escrow create --version $VERSION --include-artifacts
    - # Upload to escrow agent
  artifacts:
    paths:
      - escrow-packages/
    expire_in: 1 year
```

## Escrow Agent Integration

### Supported Agents

**Iron Mountain**
- Industry leader in information management
- Physical and digital escrow services
- 24/7 global access

**NCC Group**
- Specialized in technology escrow
- Source code verification services
- Independent testing and verification

**SES (Software Escrow Services)**
- Technology-focused escrow provider
- Rapid release mechanisms
- Cloud-based access

### API Integration

```rust
use costpilot::engines::escrow::{ReleaseAutomation, ReleaseConfig, EscrowAgentConfig};

// Configure escrow agent
let config = ReleaseConfig {
    escrow_agent: Some(EscrowAgentConfig {
        agent_name: "Iron Mountain".to_string(),
        agent_contact: "escrow@ironmountain.com".to_string(),
        agreement_id: "IM-2024-12345".to_string(),
        api_endpoint: Some("https://api.ironmountain.com/v1/deposits".to_string()),
        api_key: Some(std::env::var("ESCROW_API_KEY").unwrap()),
    }),
    ..Default::default()
};

// Create and deposit package
let automation = ReleaseAutomation::new(config, repository_root);
let package = automation.create_package("1.0.0")?;
let receipt = automation.deposit_package(&package)?;
```

## Recovery Playbook

The recovery playbook is a detailed, step-by-step guide generated for each escrow package. It includes:

### Environment Requirements
- Exact Rust and Cargo versions
- Node.js version (for VS Code extension)
- Operating system and architecture
- Required tools (git, etc.)

### Build Steps
Numbered, sequential instructions:
```markdown
### Step 1: Install Rust toolchain

```bash
cd .
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Expected:** Exit code 0
**Timeout:** 300 seconds

### Step 2: Build release binary

```bash
cd .
cargo build --release
```

**Expected:** Exit code 0
**Timeout:** 600 seconds
```

### Testing
Commands to verify the build:
```bash
cargo test
cargo test --target wasm32-unknown-unknown
```

### Expected Outputs
List of files that should exist after successful build:
- `target/release/costpilot`
- `target/wasm32-unknown-unknown/release/costpilot.wasm`

### Deployment
Instructions for deploying to production:
1. Copy binaries to deployment location
2. Verify version
3. Run initial tests
4. Configure for environment

### Troubleshooting
Common issues and solutions:
- Build failures: Check Rust version, internet connectivity
- Test failures: Verify dependencies, system resources
- Deployment issues: Check permissions, environment variables

### Support Contacts
- Vendor information
- Escrow agent contact
- Technical support resources

## Security Considerations

### Package Integrity

**Checksums:**
- SHA-256 for all source files
- Package-level checksum for overall integrity
- Verification at every stage

**Code Signing (Optional):**
- GPG signature for package
- Public key for verification
- Chain of trust from release to deployment

### Access Control

**Escrow Agent Access:**
- Only releases package upon verified trigger
- Multi-party authorization required
- Audit trail of all access

**Customer Access:**
- Limited to authorized personnel
- Requires trigger condition verification
- Time-limited access windows

### Data Privacy

**No Sensitive Data:**
- No customer data in escrow package
- No production credentials or secrets
- Only generic build instructions

**Customer Separation:**
- Customer-specific packages available
- Separate trigger conditions
- Independent release authorization

## Compliance

### SOC 2 Type II
Software escrow demonstrates:
- **Availability:** Business continuity planning
- **Confidentiality:** Protected intellectual property access
- **Processing Integrity:** Verified build process

### ISO 27001
Escrow aligns with:
- **A.17.1.1:** Planning information security continuity
- **A.17.1.2:** Implementing information security continuity
- **A.17.2.1:** Availability of information processing facilities

### GDPR
Escrow package contains:
- No personal data
- Only source code and build instructions
- Compliant with data minimization principles

## Cost & Licensing

### Escrow Service Costs

**Setup Fees:**
- Initial deposit: $2,500 - $10,000
- Agreement preparation: $1,000 - $5,000
- Verification testing: $2,000 - $15,000

**Annual Maintenance:**
- Storage and administration: $1,000 - $5,000/year
- Updates and re-deposits: $500 - $2,000 per update
- Verification testing: $1,000 - $5,000 per update

**Release Fees:**
- Trigger verification: $5,000 - $15,000
- Release administration: $2,000 - $10,000
- Technical support: $10,000 - $50,000

### Enterprise Licenses

CostPilot enterprise licenses include:
- Escrow agreement setup
- Quarterly deposit updates
- Annual verification testing
- Priority release support

## Best Practices

### For Vendors

1. **Automate Deposits:** Use CI/CD to create escrow packages on every release
2. **Regular Verification:** Test recovery process quarterly
3. **Keep Current:** Update deposits within 30 days of release
4. **Document Changes:** Maintain detailed changelog for each deposit
5. **Test Recovery:** Periodically verify packages can be built

### For Customers

1. **Verify Triggers:** Ensure trigger conditions match business needs
2. **Review Deposits:** Confirm deposits are current (within 90 days)
3. **Test Access:** Annually verify ability to access escrow
4. **Maintain Contacts:** Keep escrow agent contacts current
5. **Plan Recovery:** Document internal recovery procedures

### For Escrow Agents

1. **Independent Verification:** Test build process for each deposit
2. **Secure Storage:** Multiple geographically distributed copies
3. **Fast Release:** Have processes for rapid release upon trigger
4. **Technical Support:** Provide expert assistance during recovery
5. **Audit Trail:** Maintain complete logs of all access and releases

## FAQ

**Q: How often should escrow deposits be updated?**
A: Best practice is to deposit every major and minor release, ideally within 30 days. Critical security patches should be deposited immediately.

**Q: What happens if the build fails during recovery?**
A: The escrow package includes build verification data. If the build fails, contact the escrow agent for technical support or request vendor assistance (if available).

**Q: Can we test the escrow package without triggering release?**
A: Yes. Escrow agreements typically allow annual verification testing without triggering release conditions.

**Q: What if dependencies are no longer available?**
A: Escrow packages include cached dependencies. The `Cargo.lock` file pins exact versions, ensuring reproducible builds even if crates.io packages are removed.

**Q: How long does recovery take?**
A: Complete recovery (unpack, build, test, deploy) typically takes 2-4 hours. Emergency deployments can be prioritized with escrow agent support.

**Q: What about SaaS deployments?**
A: Escrow packages include complete source code. Customers can deploy to their own infrastructure or cloud accounts using the included build and deployment instructions.

**Q: Are third-party dependencies included?**
A: Yes. The dependencies manifest lists all Cargo and NPM dependencies with versions, checksums, and licenses. Cached copies are included where licensing permits.

**Q: What if we need customizations?**
A: The escrow package provides the complete source code. Once released, customers have full rights to modify and maintain the software according to the license terms.

## Conclusion

Software escrow transforms CostPilot from a black-box SaaS solution into a trustworthy, transparent enterprise platform. By providing complete source code access under defined trigger conditions, we demonstrate commitment to customer success and business continuity.

**Key Benefits:**
- ✅ **Risk Mitigation:** Protection against vendor failure
- ✅ **Compliance:** Meets enterprise procurement requirements
- ✅ **Transparency:** Full visibility into product internals
- ✅ **Business Continuity:** Guaranteed access to critical software
- ✅ **Trust:** Demonstrates vendor commitment to customers

For questions about software escrow or to establish an escrow agreement, contact:
- **Sales:** sales@costpilot.io
- **Support:** support@costpilot.io
- **Legal:** legal@costpilot.io
