// Diagnostics provider for cost issues

import * as vscode from 'vscode';

export class CostPilotDiagnostics {
    private diagnosticCollection: vscode.DiagnosticCollection;

    constructor(private context: vscode.ExtensionContext) {
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('costpilot');
        context.subscriptions.push(this.diagnosticCollection);
    }

    async updateFromScanResult(scanResult: any): Promise<void> {
        this.diagnosticCollection.clear();

        if (!scanResult.issues || scanResult.issues.length === 0) {
            return;
        }

        const diagnosticsMap = new Map<string, vscode.Diagnostic[]>();

        for (const issue of scanResult.issues) {
            const uri = vscode.Uri.file(issue.file);
            const diagnostics = diagnosticsMap.get(issue.file) || [];

            const range = new vscode.Range(
                new vscode.Position(issue.line - 1, issue.column || 0),
                new vscode.Position(issue.line - 1, (issue.column || 0) + (issue.length || 100))
            );

            const severity = this.mapSeverity(issue.severity);
            const diagnostic = new vscode.Diagnostic(
                range,
                issue.message,
                severity
            );

            diagnostic.source = 'CostPilot';
            diagnostic.code = issue.code;

            // Add fix if available
            if (issue.fix) {
                (diagnostic as any).fix = issue.fix;
            }

            // Add cost estimate to diagnostic
            if (issue.estimatedCost) {
                diagnostic.message += ` (Est. $${issue.estimatedCost}/month)`;
            }

            diagnostics.push(diagnostic);
            diagnosticsMap.set(issue.file, diagnostics);
        }

        // Apply all diagnostics
        for (const [file, diagnostics] of diagnosticsMap.entries()) {
            this.diagnosticCollection.set(vscode.Uri.file(file), diagnostics);
        }
    }

    private mapSeverity(severity: string): vscode.DiagnosticSeverity {
        const config = vscode.workspace.getConfiguration('costpilot');
        const defaultSeverity = config.get<string>('diagnostics.severity', 'warning');

        switch (severity.toLowerCase()) {
            case 'critical':
            case 'high':
                return vscode.DiagnosticSeverity.Error;
            case 'medium':
                return vscode.DiagnosticSeverity.Warning;
            case 'low':
                return vscode.DiagnosticSeverity.Information;
            default:
                return defaultSeverity === 'error' 
                    ? vscode.DiagnosticSeverity.Error
                    : vscode.DiagnosticSeverity.Warning;
        }
    }

    clear(): void {
        this.diagnosticCollection.clear();
    }
}
