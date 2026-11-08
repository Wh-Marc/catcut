#!/usr/bin/env bash
set -euo pipefail

echo "==> Building CatCut Frontend..."
(cd ui && pnpm install && pnpm build)

echo "==> Building CatCut Release Binary..."
cargo build --release --bin catcut

echo "==> Build finished! Binary located at: target/release/catcut"
