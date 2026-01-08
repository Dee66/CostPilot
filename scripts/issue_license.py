#!/usr/bin/env python3
"""
CostPilot License Issuer - Interactive Script

Generates cryptographically signed licenses for CostPilot customers.
Uses the license-issuer binary (must be built first).

IMPORTANT: Before using, ensure binary is rebuilt with master key:
    bash scripts/rebuild_with_master_key.sh

Usage: python3 scripts/issue_license.py
"""
import subprocess
import sys
import os
from datetime import datetime, timedelta
from pathlib import Path
import json


def run_command(cmd, cwd=None):
    """Run command and return (success, stdout, stderr)"""
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            check=False
        )
        return result.returncode == 0, result.stdout, result.stderr
    except Exception as e:
        return False, "", str(e)


def check_binary():
    """Check if license-issuer binary exists"""
    binary_path = Path("target/release/license-issuer")
    if not binary_path.exists():
        binary_path = Path("target/debug/license-issuer")

    if not binary_path.exists():
        print("❌ Error: license-issuer binary not found")
        print("\nPlease build it first:")
        print("  cargo build --release --bin license-issuer")
        sys.exit(1)

    return binary_path


def check_keypair(key_name="costpilot_master"):
    """Check if master keypair exists"""
    private_key = Path(f"{key_name}.pem")
    public_key = Path(f"{key_name}.pub.pem")

    return private_key.exists() and public_key.exists(), private_key, public_key


def generate_keypair(binary, key_name):
    """Generate new master keypair"""
    print(f"\n📝 Generating new master keypair: {key_name}")

    success, stdout, stderr = run_command([str(binary), "generate-key", key_name])

    if not success:
        print(f"❌ Keypair generation failed: {stderr}")
        sys.exit(1)

    print(f"✅ Keypair generated successfully")
    print(stdout)
    return Path(f"{key_name}.pem"), Path(f"{key_name}.pub.pem")


def validate_email(email):
    """Basic email validation"""
    return "@" in email and "." in email.split("@")[1]


def validate_date(date_str):
    """Validate RFC3339 date format"""
    try:
        datetime.fromisoformat(date_str.replace("Z", "+00:00"))
        return True
    except:
        return False


def generate_license_key():
    """Generate a random license key"""
    import random
    import string
    chars = string.ascii_uppercase + string.digits
    return ''.join(random.choices(chars, k=16))


def main():
    print("=" * 70)
    print("CostPilot License Issuer")
    print("=" * 70)
    print()
    print("⚠️  REMINDER: Ensure CostPilot binary is built with master key:")
    print("   bash scripts/rebuild_with_master_key.sh")
    print()

    # Check binary
    binary = check_binary()
    print(f"✅ Found license-issuer binary: {binary}")

    # Check master keypair
    key_name = "costpilot_master"
    has_keypair, private_key, public_key = check_keypair(key_name)

    if not has_keypair:
        print(f"\n⚠️  Master keypair not found: {key_name}.pem / {key_name}.pub.pem")
        response = input("Generate new master keypair? (yes/no): ").strip().lower()
        if response != "yes":
            print("❌ Cannot proceed without master keypair")
            sys.exit(1)
        private_key, public_key = generate_keypair(binary, key_name)
    else:
        print(f"✅ Master keypair found:")
        print(f"   Private: {private_key}")
        print(f"   Public:  {public_key}")

    print("\n" + "=" * 70)
    print("License Information")
    print("=" * 70)

    # Prompt for customer email
    while True:
        email = input("\nCustomer email: ").strip()
        if validate_email(email):
            break
        print("❌ Invalid email format. Please try again.")

    # Generate or enter license key
    suggested_key = generate_license_key()
    print(f"\nSuggested license key: {suggested_key}")
    license_key = input(f"License key (press Enter to use suggested): ").strip()
    if not license_key:
        license_key = suggested_key

    # Expiration date
    print("\nExpiration date:")
    print("  1. 30 days (monthly)")
    print("  2. 365 days (annual)")
    print("  3. Custom date")

    while True:
        choice = input("Choose (1/2/3): ").strip()
        if choice == "1":
            expires = (datetime.utcnow() + timedelta(days=30)).strftime("%Y-%m-%dT%H:%M:%SZ")
            break
        elif choice == "2":
            expires = (datetime.utcnow() + timedelta(days=365)).strftime("%Y-%m-%dT%H:%M:%SZ")
            break
        elif choice == "3":
            custom_date = input("Enter date (YYYY-MM-DDTHH:MM:SSZ): ").strip()
            if validate_date(custom_date):
                expires = custom_date
                break
            print("❌ Invalid date format. Use RFC3339 format (e.g., 2026-12-31T23:59:59Z)")
        else:
            print("❌ Invalid choice")

    # Issuer
    issuer = "costpilot-v1"
    custom_issuer = input(f"\nIssuer (press Enter for default '{issuer}'): ").strip()
    if custom_issuer:
        issuer = custom_issuer

    # Output filename
    default_output = f"license_{email.split('@')[0]}.json"
    output = input(f"\nOutput filename (press Enter for '{default_output}'): ").strip()
    if not output:
        output = default_output

    # Summary
    print("\n" + "=" * 70)
    print("License Summary")
    print("=" * 70)
    print(f"Email:       {email}")
    print(f"License Key: {license_key}")
    print(f"Expires:     {expires}")
    print(f"Issuer:      {issuer}")
    print(f"Output:      {output}")
    print(f"Private Key: {private_key}")
    print("=" * 70)

    confirm = input("\nGenerate this license? (yes/no): ").strip().lower()
    if confirm != "yes":
        print("❌ License generation cancelled")
        sys.exit(0)

    # Generate license
    print("\n🔐 Generating signed license...")

    cmd = [
        str(binary),
        "generate-license",
        "--email", email,
        "--license-key", license_key,
        "--expires", expires,
        "--private-key", str(private_key),
        "--issuer", issuer,
        "--output", output
    ]

    success, stdout, stderr = run_command(cmd)

    if not success:
        print(f"❌ License generation failed:")
        print(stderr)
        sys.exit(1)

    print(f"✅ License generated successfully!")
    print(stdout)

    # Verify the license file exists and is valid JSON
    output_path = Path(output)
    if not output_path.exists():
        print(f"❌ Warning: Output file not found: {output}")
        sys.exit(1)

    try:
        with open(output_path, 'r') as f:
            license_data = json.load(f)

        print("\n" + "=" * 70)
        print("Generated License (Verify)")
        print("=" * 70)
        print(json.dumps(license_data, indent=2))
        print("=" * 70)

        # Verify required fields
        required = ["email", "license_key", "expires", "signature", "issued_at", "version", "issuer"]
        missing = [f for f in required if f not in license_data]
        if missing:
            print(f"⚠️  Warning: Missing fields: {', '.join(missing)}")
        else:
            print("✅ All required fields present")

        print(f"\n📧 Delivery Instructions:")
        print(f"   1. Send {output} to customer at {email}")
        print(f"   2. Instruct customer to save as: ~/.costpilot/license.json")
        print(f"   3. Verify with: costpilot scan <plan.json>")
        print(f"   4. Customer should see premium features enabled")

    except json.JSONDecodeError as e:
        print(f"❌ Error: Generated file is not valid JSON: {e}")
        sys.exit(1)

    print("\n✅ License issuance complete!")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n❌ Cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
