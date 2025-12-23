// Inline cost annotations provider

import * as vscode from 'vscode';

export class CostAnnotationProvider {
    private decorationType: vscode.TextEditorDecorationType;

    constructor(private context: vscode.ExtensionContext) {
        const config = vscode.workspace.getConfiguration('costpilot');

        if (!config.get('costAnnotations.enabled', true)) {
            return;
        }

        // Create decoration type for cost annotations
        this.decorationType = vscode.window.createTextEditorDecorationType({
            after: {
                margin: '0 0 0 1em',
                textDecoration: 'none',
                color: new vscode.ThemeColor('editorCodeLens.foreground')
            }
        });

        context.subscriptions.push(this.decorationType);

        // Listen for active editor changes
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor) {
                this.updateAnnotations(editor);
            }
        }, null, context.subscriptions);

        // Initial update
        if (vscode.window.activeTextEditor) {
            this.updateAnnotations(vscode.window.activeTextEditor);
        }
    }

    private async updateAnnotations(editor: vscode.TextEditor): Promise<void> {
        if (editor.document.languageId !== 'terraform') {
            return;
        }

        // TODO: Parse Terraform resources and add cost annotations
        // For now, show placeholder
        const decorations: vscode.DecorationOptions[] = [];

        // Example: Add cost annotation after resource blocks
        const text = editor.document.getText();
        const resourceRegex = /resource\s+"([^"]+)"\s+"([^"]+)"/g;
        let match;

        while ((match = resourceRegex.exec(text)) !== null) {
            const position = editor.document.positionAt(match.index);
            const line = editor.document.lineAt(position.line);

            const decoration: vscode.DecorationOptions = {
                range: new vscode.Range(position.line, line.text.length, position.line, line.text.length),
                renderOptions: {
                    after: {
                        contentText: ` // Est. $X.XX/month`,
                        fontStyle: 'italic'
                    }
                }
            };

            decorations.push(decoration);
        }

        editor.setDecorations(this.decorationType, decorations);
    }
}
