// Code actions provider for cost fixes

import * as vscode from 'vscode';

export class CostPilotCodeActionProvider implements vscode.CodeActionProvider {
    public static readonly providedCodeActionKinds = [
        vscode.CodeActionKind.QuickFix
    ];

    constructor(private context: vscode.ExtensionContext) {}

    provideCodeActions(
        document: vscode.TextDocument,
        range: vscode.Range | vscode.Selection,
        context: vscode.CodeActionContext,
        token: vscode.CancellationToken
    ): vscode.CodeAction[] {
        const actions: vscode.CodeAction[] = [];

        // Get CostPilot diagnostics in range
        const costPilotDiagnostics = context.diagnostics.filter(
            d => d.source === 'CostPilot'
        );

        for (const diagnostic of costPilotDiagnostics) {
            const fix = (diagnostic as any).fix;
            if (fix) {
                const action = new vscode.CodeAction(
                    `💰 Apply CostPilot fix`,
                    vscode.CodeActionKind.QuickFix
                );
                action.diagnostics = [diagnostic];
                action.edit = new vscode.WorkspaceEdit();
                action.edit.replace(document.uri, diagnostic.range, fix);
                action.isPreferred = true;

                actions.push(action);
            }

            // Add "View explanation" action
            const explainAction = new vscode.CodeAction(
                `📖 Explain cost issue`,
                vscode.CodeActionKind.QuickFix
            );
            explainAction.command = {
                command: 'costpilot.explainIssue',
                title: 'Explain cost issue',
                arguments: [diagnostic]
            };
            actions.push(explainAction);
        }

        return actions;
    }
}
