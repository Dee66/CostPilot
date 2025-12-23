// Status bar integration

import * as vscode from 'vscode';

export class CostPilotStatusBar {
    private statusBarItem: vscode.StatusBarItem;

    constructor(private context: vscode.ExtensionContext) {
        const config = vscode.workspace.getConfiguration('costpilot');

        if (!config.get('statusBar.enabled', true)) {
            return;
        }

        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Right,
            100
        );

        this.statusBarItem.command = 'costpilot.showCostReport';
        this.statusBarItem.tooltip = 'Click to view cost report';

        context.subscriptions.push(this.statusBarItem);

        this.statusBarItem.text = '$(graph-line) CostPilot';
        this.statusBarItem.show();
    }

    updateFromScanResult(scanResult: any): void {
        if (!this.statusBarItem) {
            return;
        }

        const issueCount = scanResult.issues?.length || 0;
        const totalCost = scanResult.totalMonthlyCost || 0;

        if (issueCount > 0) {
            this.statusBarItem.text = `$(warning) CostPilot: ${issueCount} issue${issueCount === 1 ? '' : 's'}`;
            this.statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        } else {
            this.statusBarItem.text = `$(check) CostPilot: $${totalCost.toFixed(2)}/mo`;
            this.statusBarItem.backgroundColor = undefined;
        }

        this.statusBarItem.show();
    }

    hide(): void {
        this.statusBarItem?.hide();
    }

    show(): void {
        this.statusBarItem?.show();
    }
}
