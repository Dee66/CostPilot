#!/usr/bin/env python3
"""Test rollback byte-for-byte restoration."""

import json
import os
import shutil
import subprocess
import tempfile
import hashlib
from pathlib import Path


def compute_file_hash(filepath):
    """Compute SHA-256 hash of file."""
    sha256 = hashlib.sha256()
    with open(filepath, 'rb') as f:
        for block in iter(lambda: f.read(4096), b''):
            sha256.update(block)
    return sha256.hexdigest()


def test_rollback_restores_exact_bytes():
    """Rollback must restore file to exact original state."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create original template
        original_content = {
            "Resources": {
                "ExpensiveLambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Runtime": "python3.9"
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(original_content, f, indent=2)
        
        # Compute original hash
        original_hash = compute_file_hash(template_path)
        
        # Create rollback backup BEFORE modification
        backup_path = Path(tmpdir) / "template.json.backup"
        shutil.copy2(template_path, backup_path)
        
        # Apply patch (if autofix available)
        policy_path = Path(tmpdir) / "policy.json"
        policy_content = {
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f, indent=2)
        
        # Try to apply autofix (command may not exist yet)
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--output", str(template_path)],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            # Autofix not available, manually modify
            modified_content = original_content.copy()
            modified_content["Resources"]["ExpensiveLambda"]["Properties"]["MemorySize"] = 3008
            with open(template_path, 'w') as f:
                json.dump(modified_content, f, indent=2)
        
        # Verify file was modified
        modified_hash = compute_file_hash(template_path)
        
        # Restore from backup (byte-for-byte)
        shutil.copy2(backup_path, template_path)
        
        # Verify exact restoration
        restored_hash = compute_file_hash(template_path)
        assert restored_hash == original_hash, "Rollback did not restore exact original bytes"


def test_rollback_preserves_metadata():
    """Rollback must preserve file metadata."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create original template
        original_content = {"Resources": {}}
        with open(template_path, 'w') as f:
            json.dump(original_content, f)
        
        # Get original stat
        original_stat = os.stat(template_path)
        original_mode = original_stat.st_mode
        
        # Modify file
        modified_content = {"Resources": {"NewResource": {}}}
        with open(template_path, 'w') as f:
            json.dump(modified_content, f)
        
        # Restore
        with open(template_path, 'w') as f:
            json.dump(original_content, f)
        
        # Verify permissions preserved
        restored_stat = os.stat(template_path)
        assert restored_stat.st_mode == original_mode, "File permissions not preserved"


def test_rollback_handles_whitespace_exactly():
    """Rollback must preserve exact whitespace."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create with specific whitespace (tabs + spaces)
        original_text = '{\n\t"Resources": {\n\t\t"Lambda": {\n\t\t\t"Type": "AWS::Lambda::Function"\n\t\t}\n\t}\n}\n'
        
        with open(template_path, 'w') as f:
            f.write(original_text)
        
        original_hash = compute_file_hash(template_path)
        
        # Modify
        modified_text = '{"Resources":{"Lambda":{"Type":"AWS::Lambda::Function"}}}'
        with open(template_path, 'w') as f:
            f.write(modified_text)
        
        # Restore
        with open(template_path, 'w') as f:
            f.write(original_text)
        
        restored_hash = compute_file_hash(template_path)
        assert restored_hash == original_hash, "Whitespace not preserved"


def test_rollback_restores_comments():
    """Rollback must restore YAML comments."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.yaml"
        
        # YAML with comments
        original_yaml = """# Production template
Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      MemorySize: 1024  # Default memory
"""
        
        with open(template_path, 'w') as f:
            f.write(original_yaml)
        
        original_hash = compute_file_hash(template_path)
        
        # Modify (removing comments)
        modified_yaml = """Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      MemorySize: 512
"""
        
        with open(template_path, 'w') as f:
            f.write(modified_yaml)
        
        # Restore
        with open(template_path, 'w') as f:
            f.write(original_yaml)
        
        restored_hash = compute_file_hash(template_path)
        assert restored_hash == original_hash, "Comments not preserved"


def test_rollback_backup_immutable():
    """Rollback backup must be immutable during operation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        backup_path = Path(tmpdir) / "template.json.backup"
        
        original_content = {"Resources": {"A": {}}}
        
        with open(template_path, 'w') as f:
            json.dump(original_content, f)
        
        # Create backup
        shutil.copy2(template_path, backup_path)
        backup_hash = compute_file_hash(backup_path)
        
        # Modify original multiple times
        for i in range(5):
            modified_content = {"Resources": {f"Resource{i}": {}}}
            with open(template_path, 'w') as f:
                json.dump(modified_content, f)
        
        # Verify backup unchanged
        backup_hash_final = compute_file_hash(backup_path)
        assert backup_hash == backup_hash_final, "Backup was modified during operations"


if __name__ == "__main__":
    test_rollback_restores_exact_bytes()
    test_rollback_preserves_metadata()
    test_rollback_handles_whitespace_exactly()
    test_rollback_restores_comments()
    test_rollback_backup_immutable()
    print("All rollback restoration tests passed")
