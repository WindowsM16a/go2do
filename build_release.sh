#!/bin/bash
set -e

echo "📦 Starting Release Build..."
mkdir -p dist

# 1. Linux .deb
echo "🐧 Building Linux .deb package..."

if ! cargo deb --version &> /dev/null; then
    echo "⚠️ cargo-deb not found. Installing..."
    cargo install cargo-deb
fi

cd client
cargo deb
cd ..

# Move artifact
cp client/target/debian/*.deb dist/
echo "✅ Linux .deb created in dist/"

# 2. Windows .exe (Cross)
# echo "🪟 Building Windows .exe..."
# ... (Next step)
