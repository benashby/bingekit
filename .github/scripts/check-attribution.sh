#!/usr/bin/env bash
# Fails when commit messages in the given range, or the pull request text, carry AI
# attribution or a "how to test" section. Both are banned in this repo (CONTRIBUTING.md).
#   check-attribution.sh <base-sha> <head-sha>
# PR_TITLE and PR_BODY are read from the environment when set.
set -euo pipefail

base=$1 head=$2
attribution='co-authored-by:.*(claude|anthropic|openai|chatgpt|copilot|gemini|cursor|codex|\bai\b)|generated (with|by) .*(claude|chatgpt|copilot|gemini|ai\b)|🤖|\bai[- ]generated\b'
how_to_test='^#{0,6}[[:space:]]*(how to test|testing instructions|test plan)[[:space:]:]*$'
status=0

if [[ $base =~ ^0+$ ]]; then range=$head; else range="$base..$head"; fi
while read -r sha; do
  msg=$(git log -1 --format=%B "$sha")
  if grep -qiE "$attribution" <<<"$msg"; then
    echo "::error::commit $sha carries AI attribution"; status=1
  fi
  if grep -qiE "$how_to_test" <<<"$msg"; then
    echo "::error::commit $sha has a how-to-test section"; status=1
  fi
done < <(git rev-list "$range")

text="${PR_TITLE:-}"$'\n'"${PR_BODY:-}"
if grep -qiE "$attribution" <<<"$text"; then
  echo "::error::the pull request text carries AI attribution"; status=1
fi
if grep -qiE "$how_to_test" <<<"$text"; then
  echo "::error::the pull request text has a how-to-test section"; status=1
fi
exit $status
