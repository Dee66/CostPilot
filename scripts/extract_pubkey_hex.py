#!/usr/bin/env python3
"""Extract hex-encoded public key from PEM file for COSTPILOT_LICENSE_PUBKEY"""
import sys
import base64

if len(sys.argv) != 2:
    print("Usage: python3 extract_pubkey_hex.py <pubkey.pub.pem>")
    sys.exit(1)

pem_file = sys.argv[1]

with open(pem_file, 'r', encoding='utf-8') as f:
    content = f.read()

# Extract base64 part
lines = content.strip().split('\n')
b64_data = ''.join([line for line in lines if not line.startswith('-----')])

# Decode base64 to get raw bytes
raw_bytes = base64.b64decode(b64_data)

# Convert to hex
hex_output = raw_bytes.hex()

print(hex_output)
