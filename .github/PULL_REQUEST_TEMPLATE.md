## What changes

<!-- The behaviour before and after, in a sentence or two. Link the issue. -->

## How it was checked

<!-- The commands you ran and what they showed. For a change to compare,
     accept, reject or render output: the document pair, and what Microsoft
     Word does with the same input. -->

## Checklist

- [ ] `cargo fmt --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` pass
- [ ] A test beside the changed behaviour fails before the change and passes after it
- [ ] Output that Word must open was opened in Word, or `jubarte validate` passes on it
- [ ] `CHANGELOG.md` has an entry under Unreleased for a user-visible change
- [ ] Generated files were regenerated, not edited (`scripts/gen_docs.sh`)
- [ ] New files carry an SPDX header; a new fixture has its source recorded in `REUSE.toml`
- [ ] Commits are signed off (`git commit -s`)
