#!/bin/bash
set -euo pipefail

# Test script for CostPilot installation methods
echo "🧪 Testing CostPilot installation methods..."

# Test npm package structure
echo "Testing npm package..."
if [ ! -f "package.json" ]; then
    echo "❌ package.json not found"
    exit 1
fi

if [ ! -f "scripts/postinstall.js" ]; then
    echo "❌ scripts/postinstall.js not found"
    exit 1
fi

if [ ! -f "bin/costpilot" ]; then
    echo "❌ bin/costpilot not found"
    exit 1
fi

echo "✅ npm package structure OK"

# Test Python package structure
echo "Testing Python package..."
if [ ! -f "setup.py" ]; then
    echo "❌ setup.py not found"
    exit 1
fi

if [ ! -f "pyproject.toml" ]; then
    echo "❌ pyproject.toml not found"
    exit 1
fi

echo "✅ Python package structure OK"

# Test Homebrew formula
echo "Testing Homebrew formula..."
if [ ! -f "Formula/costpilot.rb" ]; then
    echo "❌ Formula/costpilot.rb not found"
    exit 1
fi

echo "✅ Homebrew formula OK"

# Test Docker setup
echo "Testing Docker setup..."
if [ ! -f "Dockerfile" ]; then
    echo "❌ Dockerfile not found"
    exit 1
fi

if [ ! -f ".dockerignore" ]; then
    echo "❌ .dockerignore not found"
    exit 1
fi

echo "✅ Docker setup OK"

# Test update scripts
echo "Testing update scripts..."
scripts=("update_homebrew.sh" "update_pypi.sh" "update_npm.sh" "update_docker.sh")

for script in "${scripts[@]}"; do
    if [ ! -x "scripts/$script" ]; then
        echo "❌ scripts/$script is not executable"
        exit 1
    fi
done

echo "✅ Update scripts OK"

echo ""
echo "🎉 All installation methods are properly configured!"
echo ""
echo "To test actual installations:"
echo "  npm:  npm pack && npm install -g ./costpilot-*.tgz"
echo "  pip:  python -m pip install -e ."
echo "  brew: brew install --build-from-source Formula/costpilot.rb"
echo "  docker: docker build -t costpilot:test ."