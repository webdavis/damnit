#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
owner=crates/dam-application/src/use_cases/last_seen.rs
readers=$(git grep -n -E '[.:](set_)?remote_snapshot\(|[.:]clear_remote_mapping\(' -- \
  'crates/dam-application/src/use_cases/*.rs' \
  ":(exclude)$owner" ':(exclude)*/tests/*' ':(exclude)*/tests.rs' ':(exclude)*_tests.rs' || true)
if [ -n "$readers" ]; then
  echo "Only $owner may read or write a remote snapshot; go through LastSeen:"
  echo "$readers"
  exit 1
fi
