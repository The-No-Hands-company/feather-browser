#!/bin/bash

# PronBrow Browser - Quick Start Script
# Checks dependencies and launches the browser

set -e  # Exit on error

echo "🌐 PronBrow Browser - Starting..."
echo "=================================="
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Not in pronbrow directory"
    echo "   Please cd to the pronbrow directory first"
    exit 1
fi

# Check if engine exists
if [ ! -d "../pronbrow-engine" ]; then
    echo "❌ Error: pronbrow-engine not found"
    echo "   Expected: ../pronbrow-engine/"
    exit 1
fi

# Check if engine binary exists
if [ ! -f "../pronbrow-engine/pronbrow_engine.linux-x64-gnu.node" ]; then
    echo "⚠️  Warning: Engine binary not found"
    echo "   Building engine..."
    cd ../pronbrow-engine
    cargo build --release --features napi 2>&1 | grep -E "(Compiling|Finished)" || true
    napi build --platform --release || true
    cd ../pronbrow
fi

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Check if EventBridge exists
if [ ! -f "src/engine-bridge/EventBridge.js" ]; then
    echo "❌ Error: EventBridge.js not found"
    echo "   Expected: src/engine-bridge/EventBridge.js"
    exit 1
fi

echo "✅ All checks passed!"
echo ""
echo "🚀 Launching PronBrow Browser..."
echo ""
echo "📍 Test URLs to try:"
echo "   • pronbrow://newtab"
echo "   • pronbrow://settings"
echo "   • file://$(pwd)/event-test.html"
echo ""
echo "🎮 Keyboard Shortcuts:"
echo "   • Ctrl+R - Reload"
echo "   • Ctrl+L - Focus URL bar"
echo "   • Alt+Left - Back"
echo "   • Alt+Right - Forward"
echo "   • F12 - DevTools (if in dev mode)"
echo ""
echo "=================================="
echo ""

# Launch based on argument
if [ "$1" = "dev" ]; then
    echo "🔧 Starting in DEVELOPMENT mode..."
    npm run start:dev
elif [ "$1" = "prod" ]; then
    echo "🚀 Starting in PRODUCTION mode..."
    npm run start:engine
else
    echo "🔧 Starting in DEVELOPMENT mode (default)..."
    echo "   Use: ./start.sh prod for production mode"
    npm run start:dev
fi
