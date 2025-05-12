#!/usr/bin/env bash
set -euo pipefail

# forward all arguments to the validator
exec codeowners-validation "$@"

