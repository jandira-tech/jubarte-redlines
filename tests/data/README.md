<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Test data provenance

## `wml_main_schema.json`

Vendored copy of the Open XML SDK machine-readable WordprocessingML main
schema for Ring 1½ schema-consistency tests (plan D2 / `SCHEMA_ORACLE_PLAN.md` W1).

- **Upstream path (this monorepo):** `../data/schemas/schemas_openxmlformats_org_wordprocessingml_2006_main.json`
- **Ultimate source:** [dotnet/Open-XML-SDK](https://github.com/dotnet/Open-XML-SDK) `data/` (MIT)
- **Do not hand-edit** the JSON; re-copy from the ooxmlsdk `data/` tree when refreshing.

The hand order tables in `src/comparer/finalize.rs` remain PowerTools-verbatim at
runtime; this file is a **cross-check oracle only**.
