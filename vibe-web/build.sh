#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Building vibe-web with wasm-pack..."
wasm-pack build --target web --out-dir pkg

echo ""
echo "Build complete!"
echo "To test: cd www && python3 -m http.server 8080"
echo "Then open http://localhost:8080"
