<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Held-out paired workflow pilot (V2-15 / #849)

The harness that drives the held-out pilot compares the current jubarte
CLI (arm A) with the v2 binary (arm B) on the same held-out tasks,
recorded model/version strings, randomized order, and a recorded seed.
The harness is identical across arms: only the binary under test
changes. The model owns choosing and implementing the requested
substantive changes; the CLI owns request correctness, effect
fidelity, and output identity; the harness owns status capture,
budgets, termination metadata, and artifact collection. A pilot run
is a single trial; nothing here claims a release-relevant effect.

## What the pilot does

```
run.py  --arm-a <A-bin> --arm-b <B-bin> --agent <agent.json>
        --runs-root <runs-dir> [--repetitions N] [--seed S]
```

For every cell of `tasks × arms × repetitions`, in randomized order:

1. Create a fresh `<runs-dir>/runs/<idx>-<task>-<arm>-<rep>/`
   directory with the task's fixture bytes, the prompt bound to this
   binary and run dir, and a copy of the task JSON.
2. Spawn the agent subprocess (a model CLI like `codex exec`,
   `pi -p`, `crush run`, or the bundled stub). The harness captures
   `return_code`, separate stdout/stderr, the actual artifact path
   and sha256 (scanned after the run, never inferred from a log
   tail), the wall clock duration, and the stop reason (`exit`,
   `timeout`, `cancelled`).
3. Append one `RunRecord` to `<runs-dir>/runs.jsonl` (atomic,
   `flock`-protected).

Then `grade.py` reads the ledger and calls arm A's
`jubarte read --json` and `jubarte validate` against every artifact
(or notes that the artifact does not exist for a refused run), and
writes one `GradeRecord` per run with three independent axes:

- **tool_correctness** — was the artifact readable and valid; for a
  refused run, did the agent agree that the requirement was outside
  its reach?
- **task_completion** — do the typed checks (text equality /
  containment / absence, revision / comment counts, expected
  no-artifact) pass? Rubric items stay `unassessed` until an
  external grader fills them.
- **claim_accuracy** — does the agent's printed
  `JUBARTE_RESULT:` marker agree with what the artifact and the
  checks actually show?

`report.py` writes a Markdown summary per arm and family, lists
right-censored runs, and refuses to make any effect-size claim
beyond what the counts show.

## What is recorded

Every `RunRecord` carries:

- `task_id`, `task_family`, `task_sha256` (a stable digest of the
  task payload)
- `arm`, `arm_binary_path`, `arm_binary_sha256`
- `agent_name`, `agent_model`, `agent_provider`, `agent_version`,
  `agent_argv` (the resolved command line)
- `repetition`, `order_index`, `seed`
- `started_at`, `ended_at`, `duration_seconds`, `timeout_seconds`
- `return_code`, `stop_reason`, `right_censored`
- `prompt_sha256`, `input_sha256`, `stdout_sha256`, `stderr_sha256`
- `declared_output_path`, `artifact_path`, `artifact_exists`,
  `artifact_sha256`, `artifact_bytes`
- `run_dir_files` (every file the run produced besides the harness's
  own `input.docx`, `prompt.md`, `task.json`, `stdout.txt`,
  `stderr.txt`)
- `claim` (the last `JUBARTE_RESULT: success|unsupported|failed`
  line in stdout, or `null`)

Every `GradeRecord` carries the per-check outcomes, the rubric items
(`unassessed` unless filled in), the claim value, the claim
accuracy, the grader binary's path and sha256, and the timestamp.
Missing artifacts and right-censored runs stay in the ledger and in
every denominator.

## The 12 held-out tasks

`tasks/` holds 12 task JSON files. Each task pins a fixture by
relative path and sha256, never by copy. The loader refuses to run
if any fixture's bytes have drifted.

The six families are:

| family | count | description |
|---|---|---|
| `exact_replace` | 2 | A single, unambiguous exact-string replacement. |
| `multi_edit` | 2 | Two disjoint edits composed in one command invocation (one task in the same paragraph, one across paragraphs). |
| `received_redline` | 2 | Explicit `accept` or `reject` decisions on existing revisions composed with a body edit. |
| `comments` | 2 | Comment thread operations: edit + reply, or add multiple comments in one transaction. |
| `template_flow` | 2 | A `rewrite` plan with the desired-text template, single or multi-paragraph. |
| `unsupported` | 2 | A genuinely-unsupported requirement the agent must report rather than fake. |

`load_tasks` requires exactly two tasks per family. The runner
requires the union to be exactly twelve cells per arm per
repetition (24 per repetition, 48 in the pilot's standard
configuration).

## Division of responsibility

From [`docs/design/cli-v2/jubarte-edit-implementation-plan.md`](../docs/design/cli-v2/jubarte-edit-implementation-plan.md)
section 6:

- **CLI / engine.** Correct targeting, declared transformations,
  preservation boundaries, useful refusals, source/output identity,
  exact artifact receipts, honest verification scope.
- **Agent.** Interpret authorized substantive requirements, choose
  explicit review decisions, construct necessary edits, assess
  remaining work, and make claims supported by results.
- **Harness.** Preserve subprocess status and streams,
  budgets / termination metadata, actual artifact collection,
  reproducible input / build identity, independent scoring.
- **Evaluation process.** Define outcomes, verify artifact / grade
  joins, adjudicate uncertain labels, distinguish legitimate refusal
  from defects, and report missing data.

This harness owns status capture, budgets, termination metadata,
and artifact collection. It does not score task semantics, choose
revision decisions, or replace rubric adjudication. A CLI flag
cannot prove an unexpressed legal obligation has been handled; the
harness cannot prove a requirement it was never given.

## Running the 48-run pilot

```
# From eval/cli-v2-pilot:
uv run python run.py \
    --arm-a <path/to/current-jubarte> \
    --arm-b <path/to/v2-jubarte> \
    --agent agents/<agent>.json \
    --runs-root /tmp/pilot-runs \
    --repetitions 2 \
    --seed 20260215

uv run python grade.py \
    --runs-root /tmp/pilot-runs \
    --tasks-dir tasks \
    --repo-root ../.. \
    --grader-binary <path/to/current-jubarte>

uv run python report.py \
    --runs-root /tmp/pilot-runs \
    --output /tmp/pilot-runs/report.md
```

The pilot writes:

- `runs-root/config.json` — the bound `RunConfig` (seed, arms,
  timeout, fail mode, agent provenance, randomized order).
- `runs-root/runs.jsonl` — one `RunRecord` per run.
- `runs-root/runs/<idx>-<task>-<arm>-<rep>/` — per-run directories
  with the fixture copy, prompt, task JSON, stdout/stderr, and any
  artifact the run wrote.
- `runs-root/grades.jsonl` — one `GradeRecord` per run.
- `runs-root/report.md` — Markdown summary.

For a 12-task × 2-arm × 2-repetition pilot this is 48 runs, ~48
artifact inspections, ~96 jubarte invocations from the grader, and
zero model calls (when paired with the bundled stub agent).

## The stub agent

`stub_agent.py` reads a `task.json`, walks `task.stub.steps`,
substitutes the same placeholders the harness hands a real agent
(`{binary}`, `{input}`, `{output}`, `{author}`, …), and prints the
task's `expected_claim` as a `JUBARTE_RESULT:` line. It does not call
a model; it only invokes the current jubarte binary the way the
task's stub describes. Tests and `--smoke` use it to exercise the
whole pipeline.

`stub-agent.json` is the agent config the tests pass to `run.py`:

```json
{
  "name": "stub-agent",
  "argv": ["{agent_root}/.venv/bin/python",
           "{agent_root}/stub_agent.py",
           "{task_file}", "{binary}", "{run_dir}"],
  "model": "stub",
  "provider": "stub",
  "version": "0.1.0"
}
```

For real model agents, point `--agent` at a config in
`agents/` (codex, pi, crush, or your own) and pass any command
template that takes the prompt and writes the artifact. The
harness captures every output the agent produces; the model owns
the choice of substantive edits.

## Running the tests

```
uv run pytest
```

The suite covers:

- pydantic model round-trips and the discriminated-union dispatch
  on the `Check` kind field
- atomic JSONL appends and `read_jsonl` on the ledger
- `render_argv` placeholder substitution and `build_order` seed
  determinism
- the grading per-check semantics, in isolation from the binary
- end-to-end `run.py → grade.py → report.py` with the stub agent
  and the real jubarte binary

The binary is built on demand via `/tmp/cargo-seq build --bin
jubarte` from the repository root when `target/debug/jubarte`
does not exist. Cargo runs go through `mbx`; never set
`CARGO_TARGET_DIR`, never run `cargo clean`, never remove
`target/`.

## Layout

```
eval/cli-v2-pilot/
├── pyproject.toml
├── README.md
├── stub_agent.py            # model-free reproducer
├── stub-agent.json          # agent config for the stub
├── run.py                   # pilot driver
├── grade.py                 # three-axis grader (thin wrapper)
├── report.py                # Markdown summary
├── agents/                  # sample agent configs (codex, pi, crush)
├── harness/                 # importable library
│   ├── models.py            # pydantic records (Task, RunRecord, GradeRecord, …)
│   ├── tasks.py             # task loader + fixture sha256 verification
│   ├── exec.py              # subprocess execution, capture, ledger append
│   ├── ledger.py            # atomic JSONL writes
│   ├── snapshot.py          # jubarte read/validate wrappers
│   └── grading.py           # three-axis grading core
├── tasks/                   # 12 held-out tasks (two per family)
└── tests/                   # pytest suite
```