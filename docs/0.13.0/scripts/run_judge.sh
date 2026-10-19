#!/usr/bin/env zsh
cd /Users/arthrod/temp/T/redline-bench
set -a; source .env.harbor; set +a
export OPENAI_API_KEY="$KILO_API_KEY"
export OPENAI_BASE_URL="$KILO_ENDPOINT"
J="openai/${JUDGE_MODEL#openai/}"
print -r -- "judge=$J"
for d in compare-new compare-old control-new control-old; do
  uv run redlinebench-rejudge --jobs reproduce_out/compare_experiment/jobs/$d \
    --judge "$J" --out reproduce_out/rejudge/compare-experiment/$d --workers 13
done
