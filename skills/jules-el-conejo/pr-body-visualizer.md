---
name: pr-body-visualizer
description: "🐇🥕 Use when Jules, el conejo, generates, audits, or updates a Pull Request description in GitHub Actions to enforce visual architecture summaries, before/after execution evidence, and deployment risk classification."
metadata:
  runner: github-actions
  brand: "Jules, el conejo 🐇🥕"
  original_credit: "Adapted from show-me by Dex Horthy (Humanlayer)"
  tools: ["gh", "git", "pyrefly", "ruff", "biome", "tsc", "cargo"]
---

# 🐇 PR Body Visualizer & Evidence Skill (`Jules, el conejo`) 🥕

🐇 Skip all preambles and keep prose brief—like a swift conejo hopping straight to the carrot. Use the repository's domain language from `GLOSSARY.md` (if present).

## 🥕 1. Template Structure

🐇 Enforce or generate the PR description using this exact structure:

```markdown
## Summary

<diagram, diff-sketch, or tree>

## Evidence

- **Before:** <screenshot/output/failing test run or static analysis failure>
- **After:** <screenshot/output/passing test run and clean static analysis>

## Merge Danger

**Door:** <one-way or two-way>

<optional: description of rollback mechanics, migrations, or state mutations>

**Blast Radius:** <one-word description>

<optional: potential ramifications of merge>
```

## 🐇 2. Section Guidelines

### 🥕 Summary

🐇 Pick the smallest visual representation that makes the key architectural or behavioral change clear. Place each visual next to the short text it supports, keeping only the calls, files, props, types, states, and boundaries needed to understand the diff. Use one or several formats as appropriate, without overwhelming the reader:

1. 🥕 **Logic or Algorithm (Pseudocode)**:
   ```text
   on(save)
     if content is unchanged
       return cached result
     write new content
     return fresh result
   ```

2. 🐇 **Runtime Control Flow (Call Tree)**:
   ```text
   submitForm
     createSession
       persistPrompt
       launchAgent
     navigateToSession
   ```

3. 🥕 **UI / Component Structure (Component Tree with Module Boundaries)**:
   ```text
   <SessionPage> (apps/example/src/routes/session.tsx)
     useSessionEvents()
     <SessionToolbar>
       <RunSkillButton> (packages/ui)
   ```

4. 🐇 **File Responsibility / Broad Refactor (Shallow File Tree)**:
   ```text
   src/
   ├── commands/       # parses user actions
   ├── sessions/       # owns session state
   └── transport/      # sends API requests
   ```

5. 🥕 **Component Interaction, Control Flow, or Data Flow (Mermaid)**:
   ```mermaid
   sequenceDiagram
       participant Runner as GitHub Action (el conejo 🐇)
       participant Tooling as Static Analysis (Pyrefly/Biome/Clippy 🥕)
       participant Agent as Review Agent
       Runner->>Tooling: execute check on PR diff
       Tooling-->>Agent: structured diagnostics
       Agent-->>Runner: post inline review & summary
   ```

6. 🐇 **Structural Delta (`diff` Sketch)** — Use `diff` blocks when the surrounding shape already exists and the focus is what changed:
   - *Component change*:
     ```diff
      <SessionPage>
        useSessionEvents()
        <SessionToolbar>
     +    <RunSkillButton/>
        <SessionTimeline>
     +    <SkillResultCard/>
     ```
   - *File-layout change*:
     ```diff
      src/
      ├── commands/
     +│   └── show-me.ts       # expands the slash command
      ├── sessions/
     -└── transport.ts
     +└── transport/
     +    ├── client.ts
     +    └── stream.ts
     ```
   - *Call-tree / call-stack change*:
     ```diff
      submitForm
        createSession
          persistPrompt
     +    expandSkillMention
          launchAgent
     -  navigateToSession
     +  navigateToSession
     +    subscribeToEvents
     ```
   - *State or control-flow change*:
     ```diff
      on(save)
     -  write content
     +  if content is unchanged
     +    return cached result
     +  write new content
     +  invalidate cache
     ```

7. 🥕 **Complete Target Shape** — Show the whole code block (TypeScript, Python, or Rust) when most of it is new, when omitted context would hide ownership/lifetimes or execution order, or when reviewers need a copyable target shape.

### 🐇 Evidence

🥕 Provide concrete, reproducible evidence that the change works by contrasting **Before** and **After** states:
- 🐇 **S-Tier (Visual Changes):** Screenshots or Playwright/Cypress visual diffs when the environment is set up for it and the change is visual.
- 🥕 **A-Tier (Execution & Static Verification):** Test runner output, benchmark deltas, and static checker outputs. Include exact failing-to-passing test outputs and verification gates:
  - **Python:** `pyrefly check --summarize-errors`, `ruff check`, `pytest`
  - **TypeScript:** `biome ci`, `tsc --noEmit`, `vitest run`
  - **Rust:** `cargo clippy --all-targets -- -D warnings`, `cargo test` / `cargo nextest run`

### 🥕 Merge Danger

- 🐇 **Door Classification:** State whether the PR is a `one-way` or `two-way` burrow door.
  - `two-way`: Cheap to hop back out via `git revert` with no persistent side effects.
  - `one-way`: Destructive actions, irreversible database migrations, public wire-protocol mutations, or persisted state format changes.
- 🥕 **Blast Radius:** Provide a single-word scope descriptor (e.g., `Isolated`, `Module`, `Service`, `Global`, `Cross-Tenant`) followed by concrete downstream ramifications (layout shift, consumer API breakages, cache invalidation storms, or schema lock contention).
