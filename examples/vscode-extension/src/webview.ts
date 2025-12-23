// Webview for cost reports and visualizations

import * as vscode from 'vscode';

export class CostReportWebview {
    private panel: vscode.WebviewPanel | undefined;

    constructor(private context: vscode.ExtensionContext) {}

    async show(): Promise<void> {
        if (this.panel) {
            this.panel.reveal();
            return;
        }

        this.panel = vscode.window.createWebviewPanel(
            'costpilotReport',
            'CostPilot Report',
            vscode.ViewColumn.Two,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        this.panel.onDidDispose(() => {
            this.panel = undefined;
        }, null, this.context.subscriptions);

        this.panel.webview.html = this.getDefaultHtml();
    }

    async showAnalysis(analysis: any): Promise<void> {
        await this.show();

        if (!this.panel) {
            return;
        }

        this.panel.webview.html = this.getAnalysisHtml(analysis);
    }

    async showDependencyMap(mermaidCode: string): Promise<void> {
        await this.show();

        if (!this.panel) {
            return;
        }

        this.panel.webview.html = this.getDependencyMapHtml(mermaidCode);
    }

    async showSLOReport(sloResult: any): Promise<void> {
        await this.show();

        if (!this.panel) {
            return;
        }

        this.panel.webview.html = this.getSLOReportHtml(sloResult);
    }

    private getDefaultHtml(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CostPilot Report</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            padding: 20px;
        }
        h1 {
            color: var(--vscode-editor-foreground);
            border-bottom: 2px solid var(--vscode-panel-border);
            padding-bottom: 10px;
        }
        .empty {
            text-align: center;
            margin-top: 50px;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <h1>💰 CostPilot Report</h1>
    <div class="empty">
        <p>No analysis data available.</p>
        <p>Run a workspace scan or analyze a Terraform plan to see results.</p>
    </div>
</body>
</html>`;
    }

    private getAnalysisHtml(analysis: any): string {
        const totalCost = analysis.totalMonthlyCost || 0;
        const issueCount = analysis.issues?.length || 0;

        let issuesHtml = '';
        if (analysis.issues && analysis.issues.length > 0) {
            issuesHtml = '<h2>Cost Issues</h2><ul>';
            for (const issue of analysis.issues) {
                const severityColor = this.getSeverityColor(issue.severity);
                issuesHtml += `
                    <li style="margin: 10px 0; padding: 10px; background: var(--vscode-editor-inactiveSelectionBackground); border-left: 4px solid ${severityColor};">
                        <strong>${issue.message}</strong><br>
                        <small>File: ${issue.file}:${issue.line}</small><br>
                        ${issue.estimatedCost ? `<span>Estimated Cost: $${issue.estimatedCost}/month</span><br>` : ''}
                        ${issue.fix ? `<code style="background: var(--vscode-textCodeBlock-background); padding: 5px; display: block; margin-top: 5px;">${this.escapeHtml(issue.fix)}</code>` : ''}
                    </li>
                `;
            }
            issuesHtml += '</ul>';
        } else {
            issuesHtml = '<p>✅ No cost issues detected!</p>';
        }

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CostPilot Analysis</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            padding: 20px;
        }
        h1, h2 {
            color: var(--vscode-editor-foreground);
            border-bottom: 2px solid var(--vscode-panel-border);
            padding-bottom: 10px;
        }
        .summary {
            display: flex;
            gap: 20px;
            margin: 20px 0;
        }
        .summary-card {
            flex: 1;
            padding: 20px;
            background: var(--vscode-editor-inactiveSelectionBackground);
            border-radius: 5px;
        }
        .summary-card h3 {
            margin: 0 0 10px 0;
            font-size: 14px;
            opacity: 0.8;
        }
        .summary-card .value {
            font-size: 32px;
            font-weight: bold;
        }
        ul {
            list-style: none;
            padding: 0;
        }
        code {
            white-space: pre-wrap;
        }
    </style>
</head>
<body>
    <h1>💰 CostPilot Analysis Report</h1>

    <div class="summary">
        <div class="summary-card">
            <h3>Estimated Monthly Cost</h3>
            <div class="value">$${totalCost.toFixed(2)}</div>
        </div>
        <div class="summary-card">
            <h3>Cost Issues Found</h3>
            <div class="value">${issueCount}</div>
        </div>
    </div>

    ${issuesHtml}
</body>
</html>`;
    }

    private getDependencyMapHtml(mermaidCode: string): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dependency Map</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <style>
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            padding: 20px;
        }
        h1 {
            color: var(--vscode-editor-foreground);
            border-bottom: 2px solid var(--vscode-panel-border);
            padding-bottom: 10px;
        }
        .mermaid {
            text-align: center;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <h1>🗺️  Dependency Map</h1>
    <div class="mermaid">
${mermaidCode}
    </div>
    <script>
        mermaid.initialize({ startOnLoad: true, theme: 'dark' });
    </script>
</body>
</html>`;
    }

    private getSLOReportHtml(sloResult: any): string {
        let violationsHtml = '';
        if (sloResult.violations && sloResult.violations.length > 0) {
            violationsHtml = '<h2>⚠️  SLO Violations</h2><ul>';
            for (const violation of sloResult.violations) {
                violationsHtml += `
                    <li style="margin: 10px 0; padding: 10px; background: var(--vscode-inputValidation-warningBackground); border-left: 4px solid #f39c12;">
                        <strong>${violation.slo}</strong><br>
                        <small>${violation.message}</small>
                    </li>
                `;
            }
            violationsHtml += '</ul>';
        } else {
            violationsHtml = '<p>✅ All SLOs are compliant!</p>';
        }

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SLO Compliance Report</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            padding: 20px;
        }
        h1, h2 {
            color: var(--vscode-editor-foreground);
            border-bottom: 2px solid var(--vscode-panel-border);
            padding-bottom: 10px;
        }
        ul {
            list-style: none;
            padding: 0;
        }
    </style>
</head>
<body>
    <h1>📊 SLO Compliance Report</h1>
    ${violationsHtml}
</body>
</html>`;
    }

    private getSeverityColor(severity: string): string {
        switch (severity?.toLowerCase()) {
            case 'critical':
            case 'high':
                return '#e74c3c';
            case 'medium':
                return '#f39c12';
            case 'low':
                return '#3498db';
            default:
                return '#95a5a6';
        }
    }

    private escapeHtml(unsafe: string): string {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }
}
