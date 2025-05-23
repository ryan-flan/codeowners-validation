#!/usr/bin/env bash
set -euo pipefail

# Set environment variables
export INPUT_CHECKS="${1:-all}"
export INPUT_PATH="${2:-.github/CODEOWNERS}"
export CODEOWNERS_THREADS="${CODEOWNERS_THREADS:-2}"

echo "::group::CODEOWNERS Validation"
echo "🔍 Running CODEOWNERS validation"
echo "📋 Checks: $INPUT_CHECKS"
echo "📄 Path: $INPUT_PATH"
echo ""

# Check if file exists
if [ ! -f "$INPUT_PATH" ]; then
    echo "::error::CODEOWNERS file not found at $INPUT_PATH"
    echo "❌ Error: CODEOWNERS file not found at $INPUT_PATH"
    echo "::endgroup::"
    exit 1
fi

# Run validation
set +e
codeowners-validation --checks "$INPUT_CHECKS" --path "$INPUT_PATH"
EXIT_CODE=$?
set -e

# Set output
echo "validation-passed=$([[ $EXIT_CODE -eq 0 ]] && echo "true" || echo "false")" >> $GITHUB_OUTPUT

echo "::endgroup::"

# Exit with the same code as the validation tool
exit $EXIT_CODE
