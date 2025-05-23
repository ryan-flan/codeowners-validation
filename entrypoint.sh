#!/usr/bin/env bash
set -euo pipefail

# GitHub Actions sets INPUT_* env vars from the 'with:' section
# Docker also passes args from action.yml as: --checks value --path value

# Use environment variables if set, otherwise parse from arguments
if [ -n "${INPUT_CHECKS:-}" ]; then
    CHECKS="$INPUT_CHECKS"
elif [ "$#" -ge 2 ] && [ "$1" = "--checks" ]; then
    CHECKS="$2"
else
    CHECKS="all"
fi

if [ -n "${INPUT_PATH:-}" ]; then
    PATH_ARG="$INPUT_PATH"
elif [ "$#" -ge 4 ] && [ "$3" = "--path" ]; then
    PATH_ARG="$4"
else
    PATH_ARG=".github/CODEOWNERS"
fi

# Set thread limit for CI
export CODEOWNERS_THREADS="${CODEOWNERS_THREADS:-2}"

echo "::group::CODEOWNERS Validation"
echo "🔍 Running CODEOWNERS validation"
echo "📋 Checks: $CHECKS"
echo "📄 Path: $PATH_ARG"
echo ""

# Check if file exists
if [ ! -f "$PATH_ARG" ]; then
    echo "::error::CODEOWNERS file not found at $PATH_ARG"
    echo "❌ Error: CODEOWNERS file not found at $PATH_ARG"
    echo "::endgroup::"
    exit 1
fi

# Run validation
set +e
codeowners-validation --checks "$CHECKS" --path "$PATH_ARG"
EXIT_CODE=$?
set -e

# Set output
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "validation-passed=$([[ $EXIT_CODE -eq 0 ]] && echo "true" || echo "false")" >> $GITHUB_OUTPUT
fi

echo "::endgroup::"

# Exit with the same code as the validation tool
exit $EXIT_CODE
