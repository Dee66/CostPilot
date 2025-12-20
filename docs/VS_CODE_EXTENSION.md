# VS Code Extension Documentation

## Architecture Overview

The CostPilot VS Code extension brings real-time AWS cost intelligence directly into the developer IDE. It integrates with the CostPilot CLI backend to provide inline diagnostics, quick fixes, cost annotations, and rich visualizations without requiring AWS credentials in the IDE.

### Key Design Principles

1. **Zero-IAM Architecture**: Extension never accesses AWS APIs directly - all analysis via CLI backend
2. **Deterministic**: Same IaC configuration always produces same analysis results
3. **Offline-Capable**: No network calls required during analysis
4. **Developer-First**: Inline feedback, one-click fixes, minimal friction
5. **Configurable**: Extensive settings for customizing behavior

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    VS Code Extension                         │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Diagnostics │  │ Code Actions │  │  Annotations │     │
│  │   Provider   │  │   Provider   │  │   Provider   │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
│         ┌──────────────────▼──────────────────┐             │
│         │     Extension Main (extension.ts)    │             │
│         │  - Command registration              │             │
│         │  - Component orchestration           │             │
│         │  - Event handling                    │             │
│         └──────────────┬───────────────────────┘             │
│                        │                                     │
│         ┌──────────────▼──────────────────┐                 │
│         │       CLI Wrapper (cli.ts)       │                 │
│         │  - Execute costpilot commands    │                 │
│         │  - Parse JSON output             │                 │
│         └──────────────┬───────────────────┘                 │
└────────────────────────┼─────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────┐
              │  CostPilot CLI   │
              │  - Terraform AST │
              │  - Cost analysis │
              │  - Policy check  │
              └──────────────────┘
```

### Component Breakdown

#### 1. Extension Main (`extension.ts`)
- **Purpose**: Entry point and orchestrator
- **Responsibilities**:
  - Initialize all components (diagnostics, annotations, status bar, tree view, webview)
  - Register 6 commands (scanWorkspace, analyzeTerraformPlan, showCostReport, applyFix, showDependencyMap, checkSLO)
  - Handle activation events (onLanguage, workspaceContains)
  - Set up auto-scan on save
  - Manage extension lifecycle

#### 2. CLI Wrapper (`cli.ts`)
- **Purpose**: Execute CostPilot CLI commands from extension
- **Key Functions**:
  - `executeCostPilotCLI(args: string[]): Promise<CLIResult>` - Spawn CLI process with args
  - `checkCLIInstalled(): Promise<boolean>` - Verify CLI is available
- **Configuration**: Reads `costpilot.cliPath` setting (default: "costpilot")
- **Output**: Parses JSON output from CLI, handles errors

#### 3. Diagnostics Provider (`diagnostics.ts`)
- **Purpose**: Display cost issues in VS Code Problems panel
- **Key Class**: `CostPilotDiagnostics`
- **Methods**:
  - `updateFromScanResult(result: any)` - Parse scan JSON, create diagnostics
  - `mapSeverity(severity: string): DiagnosticSeverity` - Map severity strings to VS Code enum
  - `clear()` - Clear all diagnostics
- **Features**:
  - Creates `Diagnostic` objects with Range, severity, message, source
  - Attaches fix metadata for quick actions
  - Appends cost estimates to diagnostic messages
  - Respects `costpilot.diagnostics.enabled` and `costpilot.diagnostics.severity` settings

#### 4. Code Actions Provider (`codeActions.ts`)
- **Purpose**: Provide quick fix suggestions
- **Key Class**: `CostPilotCodeActionProvider` implements `CodeActionProvider`
- **Methods**:
  - `provideCodeActions(document, range, context): CodeAction[]`
- **Actions**:
  - "Apply CostPilot fix" - Creates `WorkspaceEdit` to replace diagnostic range with fix
  - "Explain cost issue" - Opens command to explain issue in detail
- **Features**:
  - Sets `isPreferred: true` for fix actions (shown first)
  - Filters for source === 'CostPilot' diagnostics

#### 5. Cost Annotations Provider (`annotations.ts`)
- **Purpose**: Display inline cost estimates in editor
- **Key Class**: `CostAnnotationProvider`
- **Methods**:
  - `updateAnnotations(editor: TextEditor)` - Parse resources, create decorations
- **Features**:
  - Uses `TextEditorDecorationType` with `after` content
  - Displays "// Est. $X.XX/month" next to resources
  - Currently placeholder implementation (TODO: real cost calculation)
  - Respects `costpilot.costAnnotations.enabled` setting

#### 6. Status Bar (`statusBar.ts`)
- **Purpose**: Live cost summary in VS Code status bar
- **Key Class**: `CostPilotStatusBar`
- **Methods**:
  - `updateFromScanResult(result: any)` - Update based on scan results
  - `show() / hide()` - Control visibility
- **Display**:
  - Issue count: "$(warning) CostPilot: N issues" with warning background
  - No issues: "$(check) CostPilot: $X.XX/mo"
  - Click opens cost report
- **Position**: Right-aligned, priority 100

#### 7. Tree View (`treeView.ts`)
- **Purpose**: Hierarchical issue display in explorer sidebar
- **Key Class**: `CostPilotTreeDataProvider` implements `TreeDataProvider<CostIssueItem>`
- **Methods**:
  - `updateIssues(issues: any[])` - Update issue list, fire refresh
  - `getChildren(element?)` - Return severity groups or individual issues
  - `getSeverityIcon(severity)` - Map severity to emoji (🔴🟡🔵⚪)
- **Features**:
  - Groups issues by severity with counts
  - Click issue to navigate to file/line
  - Shows cost estimate in description
  - Tooltip with full explanation

#### 8. Webview (`webview.ts`)
- **Purpose**: Rich report display panels
- **Key Class**: `CostReportWebview`
- **Methods**:
  - `show()` - Create or reveal webview panel
  - `showAnalysis(analysis)` - Display scan results with issues, costs
  - `showDependencyMap(mermaidCode)` - Render Mermaid diagram
  - `showSLOReport(sloResult)` - Display SLO compliance report
- **Features**:
  - Uses `createWebviewPanel` with `enableScripts`, `retainContextWhenHidden`
  - HTML templates with VS Code theme integration
  - Mermaid.js for dependency visualization
  - Message passing between webview and extension

---

## Feature Deep-Dives

### Real-Time Diagnostics

**User Experience**:
1. Developer writes Terraform code with cost issue (e.g., large instance type)
2. Red squiggly line appears under problematic configuration
3. Hover shows diagnostic: "Large instance type - consider right-sizing. Est. $3,916.80/month"
4. Issue also appears in Problems panel (View → Problems)

**Implementation Flow**:
```typescript
// User saves .tf file (if auto-scan enabled) or runs "Scan Workspace" command
scanWorkspace()
  → executeCostPilotCLI(['scan', '--format', 'json'])
  → diagnostics.updateFromScanResult(result)
    → Parse JSON: { files: [{ path, issues: [{ line, column, severity, message, fix, cost }] }] }
    → For each issue:
      - Create Range(line-1, column-1, line-1, column+10)
      - Create Diagnostic(range, message, mapSeverity(severity))
      - Attach { fix, cost } to diagnostic metadata
      - diagnostic.source = 'CostPilot'
    → diagnosticCollection.set(uri, diagnostics[])
```

**Severity Mapping**:
- `critical` / `high` → `DiagnosticSeverity.Error` (red squigglies)
- `medium` → `DiagnosticSeverity.Warning` (yellow squigglies)
- `low` → `DiagnosticSeverity.Information` (blue squigglies)

**Configuration**:
```json
{
  "costpilot.diagnostics.enabled": true,
  "costpilot.diagnostics.severity": "warning"  // Minimum severity to show
}
```

### Quick Fix Code Actions

**User Experience**:
1. Hover over diagnostic
2. Click lightbulb icon 💡 or press `Ctrl+.` / `Cmd+.`
3. See "Apply CostPilot fix" and "Explain cost issue" actions
4. Select "Apply CostPilot fix"
5. Code automatically replaced with optimized version
6. File saved automatically

**Implementation Flow**:
```typescript
provideCodeActions(document, range, context)
  → Filter diagnostics: diagnostic.source === 'CostPilot' && diagnostic.range.intersection(range)
  → For each diagnostic with fix metadata:
    - Create fixAction: CodeAction("Apply CostPilot fix", QuickFix)
    - fixAction.edit = new WorkspaceEdit()
    - fixAction.edit.replace(uri, diagnostic.range, fix)
    - fixAction.isPreferred = true

    - Create explainAction: CodeAction("Explain cost issue", QuickFix)
    - explainAction.command = { command: 'costpilot.explainIssue', arguments: [diagnostic] }
  → Return actions
```

**Configuration**:
```json
{
  "costpilot.codeActions.enabled": true
}
```

### Inline Cost Annotations

**User Experience**:
1. Open Terraform file
2. See cost estimates displayed inline next to resources:
   ```hcl
   resource "aws_instance" "web" {  // Est. $163.20/month
     instance_type = "m5.xlarge"
     ami           = "ami-12345678"
   }
   ```

**Implementation Flow** (Current: Placeholder):
```typescript
updateAnnotations(editor)
  → regex = /resource\s+"([^"]+)"\s+"([^"]+)"/g
  → For each match:
    - resourceType = match[1]  // e.g., "aws_instance"
    - resourceName = match[2]  // e.g., "web"
    - estimatedCost = calculateCost(resourceType, attributes)  // TODO: implement
    - Create DecorationOptions at line end with "// Est. $X.XX/month"
  → editor.setDecorations(decorationType, decorations)
```

**TODO: Real Implementation**:
- Parse resource attributes from AST
- Call CLI to estimate cost: `costpilot estimate --resource-type aws_instance --attributes '{"instance_type":"m5.xlarge"}'`
- Cache results to avoid repeated calculations

**Configuration**:
```json
{
  "costpilot.costAnnotations.enabled": true,
  "costpilot.costAnnotations.position": "inline"  // or "gutter"
}
```

### Status Bar Integration

**User Experience**:
- Bottom-right status bar shows:
  - No issues: "✅ CostPilot: $1,234.56/mo" (green)
  - Issues detected: "⚠️ CostPilot: 5 issues" (yellow background)
- Click to open full cost report

**Implementation Flow**:
```typescript
updateFromScanResult(result)
  → issueCount = count issues
  → totalCost = sum all resource costs
  → If issueCount > 0:
      statusBarItem.text = `$(warning) CostPilot: ${issueCount} issues`
      statusBarItem.backgroundColor = new ThemeColor('statusBarItem.warningBackground')
    Else:
      statusBarItem.text = `$(check) CostPilot: $${totalCost.toFixed(2)}/mo`
      statusBarItem.backgroundColor = undefined
  → statusBarItem.show()
```

**Configuration**:
```json
{
  "costpilot.statusBar.enabled": true
}
```

### Tree View (Issue Navigator)

**User Experience**:
1. Open Explorer sidebar
2. See "COSTPILOT ISSUES" section
3. Issues grouped by severity:
   ```
   🔴 HIGH (3)
     ├─ Large instance type ($3,916.80/mo) - main.tf:10
     ├─ Unencrypted EBS volume - storage.tf:5
     └─ ...
   🟡 MEDIUM (2)
     ├─ Missing lifecycle policy - s3.tf:15
     └─ ...
   ```
4. Click issue to jump to location in code

**Implementation Flow**:
```typescript
getChildren(element?)
  → If element === undefined:  // Root level
      severityMap = groupBySeverity(issues)
      return severity group items: ["🔴 HIGH (3)", "🟡 MEDIUM (2)", "🔵 LOW (1)"]
    Else:  // Severity group selected
      return issues for that severity
        CostIssueItem(issue.message, TreeItemCollapsibleState.None)
          .description = issue.estimatedCost ? `$${issue.estimatedCost}/mo` : ''
          .tooltip = issue.explanation
          .command = { command: 'vscode.open', arguments: [uri, { selection: range }] }
```

**Configuration**: Always enabled when extension is active.

### Webview Reports

#### Analysis Report
**Displays**:
- Summary cards: Total monthly cost, Issue count
- Issue list with severity colors, file locations, cost impacts, suggested fixes

**Template**:
```html
<div class="summary">
  <div class="summary-card">
    <h3>Estimated Monthly Cost</h3>
    <div class="value">$1,234.56</div>
  </div>
  <div class="summary-card">
    <h3>Cost Issues Found</h3>
    <div class="value">5</div>
  </div>
</div>

<h2>Cost Issues</h2>
<ul>
  <li style="border-left: 4px solid #e74c3c;">
    <strong>Large instance type - consider right-sizing</strong><br>
    File: main.tf:10<br>
    Estimated Cost: $3,916.80/month<br>
    <code>instance_type = "m5.xlarge"</code>
  </li>
</ul>
```

#### Dependency Map
**Displays**: Mermaid diagram visualizing resource dependencies

**Template**:
```html
<div class="mermaid">
graph TD
  A[VPC] --> B[Subnet]
  B --> C[EC2 Instance]
  B --> D[RDS Database]
</div>
<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
<script>mermaid.initialize({ startOnLoad: true, theme: 'dark' });</script>
```

#### SLO Report
**Displays**:
- Compliant SLOs with ✅
- Violations with ⚠️, burn rate, time-to-breach

**Template**:
```html
<h2>⚠️ SLO Violations</h2>
<ul>
  <li style="background: var(--vscode-inputValidation-warningBackground);">
    <strong>monthly_cost_limit</strong><br>
    Current: $5,234.56 | Limit: $5,000.00 | Burn: 4.69%
  </li>
</ul>
```

---

## Command Reference

### 1. CostPilot: Scan Workspace
- **ID**: `costpilot.scanWorkspace`
- **Purpose**: Analyze all Terraform files in workspace for cost issues
- **Flow**:
  1. Find all `*.tf` files via `workspace.findFiles('**/*.tf')`
  2. Execute `costpilot scan --format json`
  3. Parse JSON output
  4. Update diagnostics, tree view, status bar
  5. Show progress notification
- **Shortcut**: Can be triggered via Command Palette or context menu
- **Auto-trigger**: Optional via `costpilot.autoScanOnSave` setting

### 2. CostPilot: Analyze Terraform Plan
- **ID**: `costpilot.analyzeTerraformPlan`
- **Purpose**: Deep-dive analysis of specific `.tfplan` file
- **Flow**:
  1. Prompt user to select `.tfplan` file (or use context URI)
  2. Execute `costpilot scan --plan <file> --explain --format json`
  3. Show analysis webview with detailed breakdown
- **Context Menu**: Right-click `.tfplan` in explorer
- **Use Case**: Before `terraform apply`, review cost impact

### 3. CostPilot: Show Cost Report
- **ID**: `costpilot.showCostReport`
- **Purpose**: Display full cost analysis report in webview
- **Flow**:
  1. Create or reveal webview panel
  2. Show analysis HTML with summary cards, issue list
- **Shortcut**: Click status bar item or run from Command Palette

### 4. CostPilot: Apply Fix
- **ID**: `costpilot.applyFix`
- **Purpose**: Apply cost optimization from diagnostic
- **Flow**:
  1. Get active editor and cursor position
  2. Find diagnostic at position with fix metadata
  3. Create `WorkspaceEdit` replacing diagnostic range with fix
  4. Apply edit and save document
- **Shortcut**: Usually invoked via lightbulb quick fix, but can be command

### 5. CostPilot: Show Dependency Map
- **ID**: `costpilot.showDependencyMap`
- **Purpose**: Visualize resource dependencies with Mermaid
- **Flow**:
  1. Execute `costpilot map --format mermaid`
  2. Parse Mermaid code from output
  3. Show webview with embedded Mermaid renderer
- **Use Case**: Understand resource relationships, blast radius

### 6. CostPilot: Check SLO Compliance
- **ID**: `costpilot.checkSLO`
- **Purpose**: Verify cost SLO adherence and burn rate
- **Flow**:
  1. Execute `costpilot slo burn --format json`
  2. Parse SLO result: { compliant: [], violations: [] }
  3. Show webview report with violations
  4. If violations, show warning notification
- **Use Case**: Pre-deployment check, continuous monitoring

---

## Configuration Reference

### costpilot.enabled
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Enable/disable the entire extension
- **Scope**: Window

### costpilot.autoScanOnSave
- **Type**: `boolean`
- **Default**: `false`
- **Description**: Automatically scan Terraform files when saved
- **Scope**: Window
- **Recommendation**: Enable for continuous feedback, disable if workspace is large

### costpilot.cliPath
- **Type**: `string`
- **Default**: `"costpilot"`
- **Description**: Path to CostPilot CLI executable
- **Scope**: Window
- **Examples**:
  - `"costpilot"` - Use from PATH
  - `"/usr/local/bin/costpilot"` - Absolute path
  - `"${workspaceFolder}/bin/costpilot"` - Workspace-relative

### costpilot.diagnostics.enabled
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Show cost issues in Problems panel
- **Scope**: Window

### costpilot.diagnostics.severity
- **Type**: `string`
- **Default**: `"warning"`
- **Description**: Default severity level for diagnostics
- **Enum**: `["error", "warning", "information"]`
- **Scope**: Window

### costpilot.costAnnotations.enabled
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Show inline cost estimates in editor
- **Scope**: Window

### costpilot.costAnnotations.position
- **Type**: `string`
- **Default**: `"inline"`
- **Description**: Where to display cost annotations
- **Enum**: `["inline", "gutter", "none"]`
- **Scope**: Window
- **Note**: `"gutter"` not yet implemented

### costpilot.codeActions.enabled
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Enable quick fix code actions
- **Scope**: Window

### costpilot.statusBar.enabled
- **Type**: `boolean`
- **Default**: `true`
- **Description**: Show cost summary in status bar
- **Scope**: Window

### costpilot.trace.server
- **Type**: `string`
- **Default**: `"off"`
- **Description**: Traces communication between VS Code and language server
- **Enum**: `["off", "messages", "verbose"]`
- **Scope**: Window
- **Use Case**: Debugging extension issues

---

## Developer Workflow Scenarios

### Scenario 1: New Terraform Resource
1. Developer creates new `aws_instance` resource
2. Types `instance_type = "m5.24xlarge"`
3. **Auto-scan triggers** (if enabled) or developer saves and runs "Scan Workspace"
4. **Red squiggly appears** under instance_type
5. Developer hovers: "Large instance type - consider right-sizing. Est. $3,916.80/month"
6. Developer clicks lightbulb 💡
7. Selects "Apply CostPilot fix: Use m5.xlarge instead"
8. Code updated to `instance_type = "m5.xlarge"`
9. **Status bar updates**: "✅ CostPilot: $163.20/mo" (saved $3,753.60/month!)

### Scenario 2: Terraform Plan Review
1. Developer runs `terraform plan -out=plan.tfplan`
2. Right-clicks `plan.tfplan` in explorer
3. Selects "Analyze Terraform Plan"
4. **Webview opens** showing:
   - Total cost before: $5,000/mo
   - Total cost after: $8,500/mo (+$3,500)
   - New resources: 5 (3 with high cost)
   - Changed resources: 2 (1 increasing cost)
5. Developer reviews issues in tree view
6. Applies fixes before deployment

### Scenario 3: SLO Monitoring
1. Team has SLO: `monthly_cost_limit: $5,000`
2. Developer runs "Check SLO Compliance"
3. **Webview shows violation**:
   - Current: $5,234.56
   - Limit: $5,000
   - Burn: 4.69%
   - Time to breach: Already breached
4. Developer navigates to high-cost resources in tree view
5. Applies optimizations to bring cost back under limit

### Scenario 4: Dependency Analysis
1. Developer modifying VPC configuration
2. Wants to understand blast radius
3. Runs "Show Dependency Map"
4. **Mermaid diagram shows**:
   ```
   VPC → Subnet → EC2 Instances (5)
               → RDS Databases (2)
               → Lambda Functions (3)
   ```
5. Developer understands changes will affect 10 resources
6. Proceeds cautiously or splits change

---

## Extension API Usage

### Registering Commands
```typescript
context.subscriptions.push(
  vscode.commands.registerCommand('costpilot.scanWorkspace', scanWorkspace)
);
```

### Creating Diagnostics
```typescript
const diagnosticCollection = vscode.languages.createDiagnosticCollection('costpilot');
const diagnostic = new vscode.Diagnostic(
  new vscode.Range(line, column, line, column + 10),
  'Large instance type - consider right-sizing',
  vscode.DiagnosticSeverity.Error
);
diagnostic.source = 'CostPilot';
diagnosticCollection.set(uri, [diagnostic]);
```

### Code Actions
```typescript
class CostPilotCodeActionProvider implements vscode.CodeActionProvider {
  provideCodeActions(document, range, context) {
    const actions: vscode.CodeAction[] = [];
    for (const diagnostic of context.diagnostics) {
      const action = new vscode.CodeAction(
        'Apply CostPilot fix',
        vscode.CodeActionKind.QuickFix
      );
      action.edit = new vscode.WorkspaceEdit();
      action.edit.replace(document.uri, diagnostic.range, fix);
      action.isPreferred = true;
      actions.push(action);
    }
    return actions;
  }
}
```

### Tree View
```typescript
class TreeDataProvider implements vscode.TreeDataProvider<Item> {
  private _onDidChangeTreeData = new vscode.EventEmitter<Item | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getChildren(element?: Item): Item[] {
    // Return root items or children
  }
}

const treeView = vscode.window.createTreeView('costpilot.issues', {
  treeDataProvider,
  showCollapseAll: true
});
```

### Webview
```typescript
const panel = vscode.window.createWebviewPanel(
  'costpilotReport',
  'CostPilot Report',
  vscode.ViewColumn.Two,
  {
    enableScripts: true,
    retainContextWhenHidden: true
  }
);
panel.webview.html = getHtml();
```

---

## Debugging and Troubleshooting

### Enable Extension Logging
1. Set `"costpilot.trace.server": "verbose"` in settings
2. Open Output panel (View → Output)
3. Select "CostPilot" from dropdown
4. See detailed logs of CLI execution, parsing, etc.

### Common Issues

#### "CLI not found" Error
**Symptom**: Extension shows error "CostPilot CLI not found"
**Solution**:
1. Verify CLI installed: `costpilot --version`
2. Check PATH includes CLI location
3. Or set absolute path: `"costpilot.cliPath": "/usr/local/bin/costpilot"`

#### No Diagnostics Shown
**Symptom**: Save `.tf` file, no diagnostics appear
**Solution**:
1. Check `costpilot.diagnostics.enabled: true`
2. Run "CostPilot: Scan Workspace" manually
3. Check Output panel for CLI errors
4. Verify `.tf` files are in workspace root

#### Auto-Scan Not Triggering
**Symptom**: Save file, no scan happens
**Solution**:
1. Enable: `"costpilot.autoScanOnSave": true`
2. Ensure saved file has `.tf` extension
3. Check extension is activated (look for status bar item)

#### Webview Shows "No data"
**Symptom**: Open report, see empty webview
**Solution**:
1. Run "Scan Workspace" first to generate data
2. Check CLI output in terminal: `costpilot scan --format json`
3. Verify JSON parsing in extension logs

### Debug Extension in VS Code
1. Open extension source in VS Code
2. Press F5 or Run → Start Debugging
3. New Extension Development Host window opens
4. Set breakpoints in TypeScript files
5. Trigger commands to hit breakpoints

---

## Marketplace Publishing Guide

### Prerequisites
1. Install `@vscode/vsce`: `npm install -g @vscode/vsce`
2. Create publisher account on [VS Code Marketplace](https://marketplace.visualstudio.com/manage)
3. Generate Personal Access Token (PAT) from Azure DevOps

### Build Extension
```bash
cd vscode-extension
npm install
npm run compile
npm run lint
npm test
```

### Package Extension
```bash
vsce package
# Generates: costpilot-0.1.0.vsix
```

### Test VSIX
```bash
code --install-extension costpilot-0.1.0.vsix
```

### Publish to Marketplace
```bash
vsce publish
# Or manually upload .vsix to marketplace.visualstudio.com
```

### Update Version
1. Bump version in `package.json`: `"version": "0.1.1"`
2. Update `CHANGELOG.md` with changes
3. Rebuild, package, publish

### Marketplace Listing Optimization
- **Icon**: 128x128 PNG, recognizable at small size
- **Screenshots**: Show key features (diagnostics, tree view, webview)
- **README**: Clear value proposition, GIFs/videos demonstrating usage
- **Keywords**: aws, cost, finops, terraform, cdk, cloudformation
- **Categories**: Programming Languages, Linters, Formatters

---

## Future Enhancements

### 1. Real-Time Cost Calculation
- Replace placeholder annotations with actual cost estimates
- Integrate with CostPilot pricing database
- Cache results for performance

### 2. Cost Budgets
- Set per-resource or per-file cost limits
- Show budget consumption in status bar
- Alert when approaching limits

### 3. Historical Cost Tracking
- Store scan results over time
- Show cost trends in webview
- Detect cost regressions

### 4. Team Collaboration
- Share cost reports via URL
- Export to PDF/CSV
- Integrate with Slack/Teams for notifications

### 5. Multi-Cloud Support
- Azure Resource Manager (ARM) templates
- Google Cloud Deployment Manager
- Kubernetes manifests with cloud provider annotations

### 6. AI-Powered Suggestions
- Use LLM to generate context-aware optimization suggestions
- Explain trade-offs between cost and performance
- Generate IaC code for cost-optimized alternatives

### 7. CI/CD Integration
- Pre-commit hooks to block cost violations
- Pull request comments with cost analysis
- Cost gate in deployment pipeline

---

## Contributing

See main project [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

**Extension-Specific Guidelines**:
- Follow VS Code extension best practices
- Use TypeScript strict mode
- Add tests for new features
- Update documentation
- Test on Windows, macOS, Linux

---

## License

Apache-2.0 - See [LICENSE](../LICENSE) for details.
