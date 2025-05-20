#!/usr/bin/env bash
set -euo pipefail

export INPUT_CHECKS="${1:-all}"

# Run the validator binary
echo "🔍 Running CODEOWNERS validation with checks: $INPUT_CHECKS"
exec codeowners-validation
