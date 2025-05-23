#!/usr/bin/env bash
# Prepare for a release
#
# All additional options are passed to `rooster release`
set -eu

export UV_PREVIEW=1

script_root="$(realpath "$(dirname "$0")")"
project_root="$(dirname "$script_root")"

echo "Updating metadata with rooster..."
cd "$project_root"
uvx --from 'rooster-blue>=0.0.7' --python 3.12 --isolated -- \
    rooster release "$@"

echo "Updating lockfile..."
cargo update -p fortitude

echo "Generating contributors list..."
echo ""
echo ""
uvx --from 'rooster-blue>=0.0.7' --python 3.12 --isolated -- \
    rooster contributors --quiet
