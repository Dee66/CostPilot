// VS Code Extension Entry Point for CostPilot

import * as vscode from 'vscode';
import { CostPilotDiagnostics } from './diagnostics';
import { CostAnnotationProvider } from './annotations';
import { CostPilotCodeActionProvider } from './codeActions';
import { CostPilotStatusBar } from './statusBar';
import { CostReportWebview } from './webview';
import { CostPilotTreeDataProvider } from './treeView';
import { executeCostPilotCLI } from './cli';

export function activate(context: vscode.ExtensionContext) {
    console.log('CostPilot extension activated');

    const config = vscode.workspace.getConfiguration('costpilot');
    
    if (!config.get('enabled', true)) {
        console.log('CostPilot is disabled');
        return;
    }

    // Initialize components
    const diagnostics = new CostPilotDiagnostics(context);
    const annotations = new CostAnnotationProvider(context);
    const statusBar = new CostPilotStatusBar(context);
    const treeDataProvider = new CostPilotTreeDataProvider(context);
    const reportWebview = new CostReportWebview(context);

    // Register tree view
    const treeView = vscode.window.createTreeView('costpilot.issues', {
        treeDataProvider: treeDataProvider,
        showCollapseAll: true
    });
    context.subscriptions.push(treeView);

    // Register code action provider for Terraform files
    if (config.get('codeActions.enabled', true)) {
        context.subscriptions.push(
            vscode.languages.registerCodeActionsProvider(
                { language: 'terraform', scheme: 'file' },
                new CostPilotCodeActionProvider(context),
                {
                    providedCodeActionKinds: CostPilotCodeActionProvider.providedCodeActionKinds
                }
            )
        );
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.scanWorkspace', async () => {
            await scanWorkspace(context, diagnostics, treeDataProvider, statusBar);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.analyzeTerraformPlan', async (uri?: vscode.Uri) => {
            await analyzeTerraformPlan(uri, context, reportWebview);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.showCostReport', async () => {
            await reportWebview.show();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.applyFix', async (diagnostic: vscode.Diagnostic) => {
            await applyFix(diagnostic, context);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.showDependencyMap', async () => {
            await showDependencyMap(context, reportWebview);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('costpilot.checkSLO', async () => {
            await checkSLO(context, reportWebview);
        })
    );

    // Auto-scan on save if enabled
    if (config.get('autoScanOnSave', false)) {
        context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument(async (document) => {
                if (document.languageId === 'terraform' || document.fileName.endsWith('.tf')) {
                    await scanWorkspace(context, diagnostics, treeDataProvider, statusBar);
                }
            })
        );
    }

    // Initial scan
    if (vscode.workspace.workspaceFolders) {
        scanWorkspace(context, diagnostics, treeDataProvider, statusBar);
    }

    // Show welcome message
    const hasShownWelcome = context.globalState.get('costpilot.hasShownWelcome', false);
    if (!hasShownWelcome) {
        vscode.window.showInformationMessage(
            'CostPilot activated! Run "CostPilot: Scan Workspace" to analyze your infrastructure costs.',
            'Scan Now',
            'Dismiss'
        ).then(selection => {
            if (selection === 'Scan Now') {
                vscode.commands.executeCommand('costpilot.scanWorkspace');
            }
        });
        context.globalState.update('costpilot.hasShownWelcome', true);
    }
}

async function scanWorkspace(
    context: vscode.ExtensionContext,
    diagnostics: CostPilotDiagnostics,
    treeDataProvider: CostPilotTreeDataProvider,
    statusBar: CostPilotStatusBar
): Promise<void> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showWarningMessage('No workspace folder open');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'CostPilot: Scanning workspace',
        cancellable: false
    }, async (progress) => {
        progress.report({ message: 'Analyzing Terraform files...' });

        try {
            // Find Terraform files
            const tfFiles = await vscode.workspace.findFiles('**/*.tf', '**/node_modules/**');
            
            if (tfFiles.length === 0) {
                vscode.window.showInformationMessage('No Terraform files found in workspace');
                return;
            }

            progress.report({ message: `Found ${tfFiles.length} Terraform files` });

            // Execute CostPilot scan
            const result = await executeCostPilotCLI(['scan', '--format', 'json'], workspaceFolder.uri.fsPath);
            
            if (result.success && result.output) {
                const scanResult = JSON.parse(result.output);
                
                // Update diagnostics
                await diagnostics.updateFromScanResult(scanResult);
                
                // Update tree view
                treeDataProvider.updateIssues(scanResult.issues || []);
                
                // Update status bar
                statusBar.updateFromScanResult(scanResult);
                
                const issueCount = scanResult.issues?.length || 0;
                if (issueCount > 0) {
                    vscode.window.showInformationMessage(
                        `CostPilot found ${issueCount} cost issue${issueCount === 1 ? '' : 's'}`,
                        'View Issues'
                    ).then(selection => {
                        if (selection === 'View Issues') {
                            vscode.commands.executeCommand('costpilot.issues.focus');
                        }
                    });
                } else {
                    vscode.window.showInformationMessage('✅ No cost issues found!');
                }
            } else {
                vscode.window.showErrorMessage(`CostPilot scan failed: ${result.error || 'Unknown error'}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CostPilot error: ${error}`);
        }
    });
}

async function analyzeTerraformPlan(
    uri: vscode.Uri | undefined,
    context: vscode.ExtensionContext,
    reportWebview: CostReportWebview
): Promise<void> {
    let planPath: string;

    if (uri) {
        planPath = uri.fsPath;
    } else {
        // Prompt user to select plan file
        const result = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectFolders: false,
            canSelectMany: false,
            filters: {
                'Terraform Plans': ['json', 'tfplan'],
                'All Files': ['*']
            },
            title: 'Select Terraform Plan File'
        });

        if (!result || result.length === 0) {
            return;
        }

        planPath = result[0].fsPath;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'CostPilot: Analyzing Terraform plan',
        cancellable: false
    }, async (progress) => {
        try {
            progress.report({ message: 'Running cost analysis...' });

            const result = await executeCostPilotCLI([
                'scan',
                '--plan', planPath,
                '--format', 'json',
                '--explain'
            ]);

            if (result.success && result.output) {
                const analysis = JSON.parse(result.output);
                await reportWebview.showAnalysis(analysis);
            } else {
                vscode.window.showErrorMessage(`Analysis failed: ${result.error || 'Unknown error'}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CostPilot error: ${error}`);
        }
    });
}

async function applyFix(
    diagnostic: vscode.Diagnostic,
    context: vscode.ExtensionContext
): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    // Extract fix from diagnostic metadata
    const fix = (diagnostic as any).fix;
    if (!fix) {
        vscode.window.showWarningMessage('No fix available for this issue');
        return;
    }

    const edit = new vscode.WorkspaceEdit();
    edit.replace(editor.document.uri, diagnostic.range, fix);

    const success = await vscode.workspace.applyEdit(edit);
    if (success) {
        vscode.window.showInformationMessage('✅ Cost fix applied successfully');
        await editor.document.save();
    } else {
        vscode.window.showErrorMessage('Failed to apply fix');
    }
}

async function showDependencyMap(
    context: vscode.ExtensionContext,
    reportWebview: CostReportWebview
): Promise<void> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showWarningMessage('No workspace folder open');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'CostPilot: Generating dependency map',
        cancellable: false
    }, async (progress) => {
        try {
            const result = await executeCostPilotCLI([
                'map',
                '--format', 'mermaid'
            ], workspaceFolder.uri.fsPath);

            if (result.success && result.output) {
                await reportWebview.showDependencyMap(result.output);
            } else {
                vscode.window.showErrorMessage(`Map generation failed: ${result.error || 'Unknown error'}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CostPilot error: ${error}`);
        }
    });
}

async function checkSLO(
    context: vscode.ExtensionContext,
    reportWebview: CostReportWebview
): Promise<void> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showWarningMessage('No workspace folder open');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'CostPilot: Checking SLO compliance',
        cancellable: false
    }, async (progress) => {
        try {
            const result = await executeCostPilotCLI([
                'slo', 'burn',
                '--format', 'json'
            ], workspaceFolder.uri.fsPath);

            if (result.success && result.output) {
                const sloResult = JSON.parse(result.output);
                await reportWebview.showSLOReport(sloResult);

                // Show notification if SLO violated
                if (sloResult.violations && sloResult.violations.length > 0) {
                    vscode.window.showWarningMessage(
                        `⚠️  ${sloResult.violations.length} SLO violation(s) detected`,
                        'View Report'
                    ).then(selection => {
                        if (selection === 'View Report') {
                            reportWebview.show();
                        }
                    });
                } else {
                    vscode.window.showInformationMessage('✅ All SLOs are compliant');
                }
            } else {
                vscode.window.showErrorMessage(`SLO check failed: ${result.error || 'Unknown error'}`);
            }
        } catch (error) {
            vscode.window.showErrorMessage(`CostPilot error: ${error}`);
        }
    });
}

export function deactivate() {
    console.log('CostPilot extension deactivated');
}
