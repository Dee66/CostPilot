// CLI execution wrapper for CostPilot

import { exec } from 'child_process';
import * as vscode from 'vscode';
import * as path from 'path';

export interface CLIResult {
    success: boolean;
    output?: string;
    error?: string;
    exitCode?: number;
}

export async function executeCostPilotCLI(
    args: string[],
    cwd?: string
): Promise<CLIResult> {
    const config = vscode.workspace.getConfiguration('costpilot');
    const cliPath = config.get<string>('cliPath', 'costpilot');

    const command = `${cliPath} ${args.join(' ')}`;

    return new Promise((resolve) => {
        exec(
            command,
            {
                cwd: cwd || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
                maxBuffer: 10 * 1024 * 1024 // 10MB buffer
            },
            (error, stdout, stderr) => {
                if (error) {
                    resolve({
                        success: false,
                        error: stderr || error.message,
                        exitCode: error.code
                    });
                } else {
                    resolve({
                        success: true,
                        output: stdout
                    });
                }
            }
        );
    });
}

export async function checkCLIInstalled(): Promise<boolean> {
    const result = await executeCostPilotCLI(['--version']);
    return result.success;
}
