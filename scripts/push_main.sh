#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Sourced by scripts/release.sh (the app repository keeps its own copy for
# its scripts/release-engine.sh).
#
#   push_main BRANCH TITLE BODY
#
# Puts HEAD on origin's main. A direct push first. Where main takes changes
# only through a pull request (a ruleset), HEAD goes up as BRANCH, a pull
# request is opened and merged at once with a merge commit, so the commit
# that was gated and tagged is itself in main; the local branch then
# fast-forwards to that merge. A rerun finds the open pull request and
# merges it. Fails when neither way lands.
push_main() {
  local branch="$1" title="$2" body="$3" refusal try
  if refusal=$(git push -q origin HEAD:main 2>&1); then
    return 0
  fi
  printf '  - main refused a direct push; going through a pull request (%s)\n' "$branch"
  git push -q --force origin "HEAD:refs/heads/$branch" \
    || { printf '%s\n' "$refusal" >&2; return 1; }
  if ! gh pr view "$branch" --json state --jq .state 2>/dev/null | grep -x OPEN >/dev/null; then
    gh pr create --base main --head "$branch" --title "$title" --body "$body" >/dev/null || return 1
  fi
  # GitHub computes mergeability after the pull request exists: a first
  # merge can be refused for a few seconds.
  for try in 1 2 3 4 5 6; do
    if gh pr merge "$branch" --merge >/dev/null; then
      git fetch -q origin main && git merge -q --ff-only FETCH_HEAD
      return
    fi
    [ "$try" = 6 ] || sleep "${PUSH_MAIN_RETRY_SECONDS:-5}"
  done
  return 1
}
