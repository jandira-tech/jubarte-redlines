#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""The held-out pilot runner.

`run.py` drives the harness's execution core: every (task, arm,
repetition) cell is run in a fresh directory, in a randomized order
seeded for reproducibility. Subprocess return code, separate stdout and
stderr, start and end timestamps, stop reason, the scanned artifact path
and digest, and the model/version strings are written to one JSONL
record per run, appended atomically.

The runner takes the binary path for each interface arm and the
`--repetitions` count from the command line; the agent command template
comes from the agent config in the harness (see
`harness/agents/<name>.json`). The harness never infers success from a
log tail or a shell status: each run's only delivery facts are the
artifact path, its sha256 and its size, captured by `exec.run_one`.
"""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import shutil
import sys
import tempfile
import uuid
from pathlib import Path

from harness.exec import build_order, make_config, run_one, utc_now
from harness.ledger import write_json_atomic
from harness.models import AgentConfig, ArmConfig
from harness.tasks import load_tasks

HERE = Path(__file__).resolve().parent
DEFAULT_REPO_ROOT = HERE.parent.parent


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_agent(agent_path: Path) -> AgentConfig:
    return AgentConfig.model_validate_json(agent_path.read_text(encoding="utf-8"))


def load_arms(arms: list[tuple[str, Path]]) -> list[ArmConfig]:
    out: list[ArmConfig] = []
    for label, path in arms:
        out.append(
            ArmConfig(
                label=label,  # type: ignore[arg-type]
                binary=str(path),
                sha256=sha256_file(path),
                role=f"jubarte CLI binary {label}",
            )
        )
    return out


async def main(args: argparse.Namespace) -> int:
    repo_root = args.repo_root.resolve()
    tasks_dir = args.tasks_dir.resolve()
    agent_path = args.agent.resolve()
    runs_root = args.runs_root.resolve()

    if args.smoke:
        # Smoke: a single task, one repetition, one arm — for the --smoke
        # pipeline test that uses the stub agent.
        args.repetitions = 1

    if runs_root.exists():
        shutil.rmtree(runs_root)
    runs_root.mkdir(parents=True)

    tasks = load_tasks(tasks_dir, repo_root)
    agent = load_agent(agent_path)
    arms = load_arms([("A", args.arm_a.resolve()), ("B", args.arm_b.resolve())])
    order = build_order(tasks, arms, args.repetitions, args.seed)
    config = make_config(
        pilot_id=f"pilot-{uuid.uuid4().hex[:8]}",
        seed=args.seed,
        repetitions=args.repetitions,
        timeout_seconds=args.timeout_seconds,
        fail_mode=args.fail_mode,
        agent=agent,
        arms=arms,
        order=order,
    )
    write_json_atomic(runs_root / "config.json", config.model_dump(mode="json"))

    binary_by_label = {arm.label: Path(arm.binary) for arm in arms}
    agent_root = agent_path.resolve().parent
    for entry in order:
        task = tasks[entry.task_id]
        arm_cfg = next(a for a in arms if a.label == entry.arm)
        binary = binary_by_label[entry.arm]
        try:
            await run_one(
                task=task,
                arm=arm_cfg,
                agent=agent,
                repetition=entry.repetition,
                order_index=entry.order_index,
                seed=args.seed,
                timeout_seconds=args.timeout_seconds,
                fail_mode=args.fail_mode,
                runs_root=runs_root,
                pilot_id=config.pilot_id,
                binary=binary,
                agent_root=agent_root,
            )
        except asyncio.CancelledError:
            raise
        except Exception as exc:
            print(
                f"harness: run {entry.order_index} ({entry.task_id} arm={entry.arm}) "
                f"failed unexpectedly: {exc}",
                file=sys.stderr,
            )
    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="run.py",
        description="Run the held-out paired workflow pilot.",
    )
    parser.add_argument("--repo-root", type=Path, default=DEFAULT_REPO_ROOT)
    parser.add_argument("--tasks-dir", type=Path, default=HERE / "tasks")
    parser.add_argument("--agent", type=Path, required=True,
                        help="Path to the agent config JSON.")
    parser.add_argument("--arm-a", type=Path, required=True,
                        help="Path to the current jubarte CLI binary (arm A).")
    parser.add_argument("--arm-b", type=Path, required=True,
                        help="Path to the v2 jubarte CLI binary (arm B).")
    parser.add_argument("--runs-root", type=Path,
                        default=Path(tempfile.gettempdir()) / "pilot-runs",
                        help="Directory the pilot writes its ledger into.")
    parser.add_argument("--seed", type=int, default=20260215)
    parser.add_argument("--repetitions", type=int, default=2)
    parser.add_argument("--timeout-seconds", type=float, default=120.0)
    parser.add_argument("--fail-mode", default="real",
                        help="Mode tag passed through to the agent command template.")
    parser.add_argument("--smoke", action="store_true",
                        help="Run a single repetition with one task for smoke tests.")
    return parser.parse_args(argv)


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main(parse_args(sys.argv[1:]))))