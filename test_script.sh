#!/bin/bash
set -euo pipefail

echo "Test script"

test_function() {
    echo "In test function"
}

case "${1:-}" in
    test)
        test_function
        ;;
    *)
        echo "Usage: $0 test"
        ;;
esac
