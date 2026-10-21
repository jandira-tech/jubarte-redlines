# PlanV2 conformance fixtures

Copied byte-for-byte from docs/design/cli-v2/examples in the V2 design worktree,
2026-10-17. The five template plans and example-source.docx are unchanged.
malformed-plans.json materializes all 16 PlanV2 malformed-schema mutations from
validate_examples.py. Its final three malformed cases concern the separate
expectations schema and belong to the later expectations issue.

The production parser validates request shape. Pair identity, source binding,
working hashes, scope and composition require the later source compiler.
