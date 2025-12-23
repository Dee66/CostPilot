import * as assert from 'assert';
import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
    vscode.window.showInformationMessage('Start all tests.');

    test('Extension should be present', () => {
        assert.ok(vscode.extensions.getExtension('guardsuite.costpilot'));
    });

    test('Extension should activate', async () => {
        const ext = vscode.extensions.getExtension('guardsuite.costpilot');
        assert.ok(ext);
        await ext.activate();
        assert.strictEqual(ext.isActive, true);
    });

    test('Commands should be registered', async () => {
        const commands = await vscode.commands.getCommands(true);

        assert.ok(commands.includes('costpilot.scanWorkspace'));
        assert.ok(commands.includes('costpilot.analyzeTerraformPlan'));
        assert.ok(commands.includes('costpilot.showCostReport'));
        assert.ok(commands.includes('costpilot.applyFix'));
        assert.ok(commands.includes('costpilot.showDependencyMap'));
        assert.ok(commands.includes('costpilot.checkSLO'));
    });
});
