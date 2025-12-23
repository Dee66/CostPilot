# CostPilot VS Code Extension

Bring real-time AWS cost intelligence directly into your IDE. CostPilot analyzes Terraform and AWS CDK configurations to detect cost issues as you code, with inline diagnostics, quick fixes, and rich visualizations.

## 🚀 Features

- **Real-time Cost Diagnostics**: Catch cost issues as you write Terraform code
- **Inline Cost Estimates**: See estimated monthly costs next to resources
- **Quick Fixes**: One-click cost optimizations right in your editor
- **Cost Issue Tree View**: Organize and navigate issues by severity
- **Status Bar Summary**: Continuous cost awareness while you code
- **Rich Reports**: Interactive cost analysis with dependency maps
- **SLO Compliance Checks**: Ensure your infrastructure meets cost objectives
- **Terraform Plan Analysis**: Deep-dive into plan files before applying

## 📦 Installation

1. Install [CostPilot CLI](https://github.com/guardsuite/costpilot) (required):
   ```bash
   cargo install costpilot
   ```

2. Install this extension from the VS Code Marketplace

3. Open a workspace containing Terraform files (`.tf`) or AWS CDK projects (`cdk.json`)

4. The extension will activate automatically!

## 🔧 Configuration

Configure CostPilot through VS Code settings:

```json
{
  // Enable/disable the extension
  "costpilot.enabled": true,

  // Auto-scan on save (default: off)
  "costpilot.autoScanOnSave": false,

  // Path to CostPilot CLI executable
  "costpilot.cliPath": "costpilot",

  // Diagnostics settings
  "costpilot.diagnostics.enabled": true,
  "costpilot.diagnostics.severity": "warning",

  // Cost annotations settings
  "costpilot.costAnnotations.enabled": true,
  "costpilot.costAnnotations.position": "inline",

  // Code actions (quick fixes)
  "costpilot.codeActions.enabled": true,

  // Status bar summary
  "costpilot.statusBar.enabled": true
}
```

## 🎯 Usage

### Commands

Access commands via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

- **CostPilot: Scan Workspace** - Analyze all Terraform files for cost issues
- **CostPilot: Analyze Terraform Plan** - Deep-dive into a `.tfplan` file
- **CostPilot: Show Cost Report** - View detailed cost analysis report
- **CostPilot: Apply Fix** - Apply cost optimization from diagnostic
- **CostPilot: Show Dependency Map** - Visualize resource dependencies
- **CostPilot: Check SLO Compliance** - Verify cost SLO adherence

### Context Menus

Right-click on:
- **`.tf` files** in editor → Scan for Cost Issues
- **`.tfplan` files** in explorer → Analyze Terraform Plan

### Status Bar

Click the CostPilot status bar item to view the full cost report:
- ✅ Shows total estimated monthly cost when no issues
- ⚠️ Shows issue count when problems detected

### Tree View

Explore cost issues in the Explorer sidebar:
- Issues grouped by severity (🔴 HIGH, 🟡 MEDIUM, 🔵 LOW)
- Click any issue to jump to its location in code
- Shows estimated cost impact per issue

### Quick Fixes

1. Hover over a diagnostic (squiggly line)
2. Click the lightbulb icon 💡
3. Select "Apply CostPilot fix" for one-click optimization

## 📊 Example Workflow

1. **Write Terraform code**:
   ```hcl
   resource "aws_instance" "web" {
     instance_type = "m5.24xlarge"  // 🔴 Diagnostic appears
     ami           = "ami-12345678"
   }
   ```

2. **See inline diagnostic**: "Large instance type - consider right-sizing"

3. **View cost estimate**: `// Est. $3,916.80/month` appears inline

4. **Apply quick fix**: Click lightbulb → "Use m5.xlarge instead"

5. **Check status bar**: "✅ CostPilot: $163.20/mo" (savings applied!)

## 🛠️ Requirements

- **VS Code**: 1.85.0 or higher
- **CostPilot CLI**: Latest version installed and in PATH
- **Workspace**: Must contain Terraform (`.tf`) or CDK (`cdk.json`) files

## 🐛 Troubleshooting

### Extension doesn't activate
- Check you have `.tf` files or `cdk.json` in your workspace
- Verify CostPilot CLI is installed: `costpilot --version`

### "CLI not found" error
- Ensure CostPilot CLI is in your PATH
- Or set `costpilot.cliPath` to absolute path: `/usr/local/bin/costpilot`

### No diagnostics shown
- Run "CostPilot: Scan Workspace" command manually
- Check diagnostics are enabled: `costpilot.diagnostics.enabled: true`
- Verify you have Terraform files in the workspace

### Auto-scan not working
- Enable it: `costpilot.autoScanOnSave: true`
- Save a `.tf` file to trigger scan

## 📖 Documentation

- [CostPilot CLI Documentation](https://github.com/guardsuite/costpilot)
- [Extension Architecture](docs/VS_CODE_EXTENSION.md)
- [Contributing Guide](CONTRIBUTING.md)

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📝 License

Apache-2.0 - See [LICENSE](../LICENSE) for details.

## 🔗 Links

- [GitHub Repository](https://github.com/guardsuite/costpilot)
- [Report Issues](https://github.com/guardsuite/costpilot/issues)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=guardsuite.costpilot)

---

**Made with ❤️ by GuardSuite** | Deterministic AWS cost intelligence for the paranoid
