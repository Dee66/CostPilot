// Tree view for cost issues

import * as vscode from 'vscode';

export class CostPilotTreeDataProvider implements vscode.TreeDataProvider<CostIssueItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<CostIssueItem | undefined | null | void> = new vscode.EventEmitter<CostIssueItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<CostIssueItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private issues: any[] = [];

    constructor(private context: vscode.ExtensionContext) {}

    updateIssues(issues: any[]): void {
        this.issues = issues;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: CostIssueItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: CostIssueItem): Thenable<CostIssueItem[]> {
        if (element) {
            return Promise.resolve([]);
        }

        if (this.issues.length === 0) {
            return Promise.resolve([]);
        }

        // Group issues by severity
        const grouped = new Map<string, any[]>();
        for (const issue of this.issues) {
            const severity = issue.severity || 'medium';
            const issues = grouped.get(severity) || [];
            issues.push(issue);
            grouped.set(severity, issues);
        }

        const items: CostIssueItem[] = [];

        // Add severity groups
        for (const [severity, issues] of grouped.entries()) {
            const item = new CostIssueItem(
                `${this.getSeverityIcon(severity)} ${severity.toUpperCase()} (${issues.length})`,
                vscode.TreeItemCollapsibleState.Expanded
            );
            item.contextValue = 'severityGroup';
            items.push(item);

            // Add individual issues
            for (const issue of issues) {
                const issueItem = new CostIssueItem(
                    issue.message,
                    vscode.TreeItemCollapsibleState.None
                );
                issueItem.description = issue.estimatedCost ? `$${issue.estimatedCost}/mo` : '';
                issueItem.tooltip = issue.explanation || issue.message;
                issueItem.command = {
                    command: 'vscode.open',
                    title: 'Go to issue',
                    arguments: [
                        vscode.Uri.file(issue.file),
                        {
                            selection: new vscode.Range(
                                new vscode.Position(issue.line - 1, issue.column || 0),
                                new vscode.Position(issue.line - 1, (issue.column || 0) + (issue.length || 100))
                            )
                        }
                    ]
                };
                issueItem.contextValue = 'costIssue';
                items.push(issueItem);
            }
        }

        return Promise.resolve(items);
    }

    private getSeverityIcon(severity: string): string {
        switch (severity.toLowerCase()) {
            case 'critical':
            case 'high':
                return '🔴';
            case 'medium':
                return '🟡';
            case 'low':
                return '🔵';
            default:
                return '⚪';
        }
    }
}

class CostIssueItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);
    }
}
