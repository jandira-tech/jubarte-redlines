# Symbol manifest for `src/`

Every function, method, struct, enum, trait, impl block, constant, static, type alias, macro and module under `src/`, extracted statically with tree-sitter (Rust grammar). No LLM agents and no compiler were involved. Regenerate with `scripts/gen_symbol_manifest.py` (see the top of that file).

## What is in here

| Manifest | Items |
|---|---:|
| [functions/](functions/) (one file per source file; functions and methods) | 5677 non-test + 4004 test |
| [structs.md](structs.md) | 607 |
| [enums.md](enums.md) (enums and unions) | 236 |
| [traits.md](traits.md) | 5 |
| [impls.md](impls.md) | 388 |
| [consts.md](consts.md) (consts and statics) | 726 |
| [type_aliases.md](type_aliases.md) | 86 |
| [macros.md](macros.md) (`macro_rules!`) | 3 |
| [modules.md](modules.md) | 155 |
| [symbols.json](symbols.json) | everything above, including test items and every call edge |

## How to read a function entry

- **Does** is the item's own `///` doc comment (first sentence). If there is none, a plain `//` comment directly above it is used and marked as such. Otherwise it says _no doc comment_: nothing is guessed. Coverage is below.
- **Calls (crate)** lists the crate functions this one calls, as `qualified name (file:line)` and the call-site lines. **Calls (std/external/unresolved)** lists paths that did not resolve to a crate function (standard library, dependencies, enum variants, tuple-struct constructors). **Macros** lists macro invocations.
- **Called by** is the inverse of the crate call edges.

### Call-resolution confidence

tree-sitter gives syntax, not types, so a call is matched to a definition by name:

- no tag: unique match found by a rule that is safe without types (same-file free function, `Self::f` / `Type::f` with a matching impl, `module::f` with a matching module file, `self.f()` on the same impl type).
- _(by name)_: a method or free function with that name exists exactly once in the crate and nothing better matched. A `.len()` on a std type would be mis-attributed if the crate defined one `len` method, so read these as likely, not certain.
- _ambiguous_: several crate definitions share the name; candidates are listed (or counted when more than four). In **Called by**, a `(?)` marks edges of this kind.
- calls through trait objects, generics, closures stored in variables, function pointers and `.map(Self::f)`-style references are not seen as calls.
- calls inside macro arguments (`assert_eq!(f(x), y)`, `format!`) are recovered from the token tree by pattern (identifier followed by parentheses), which is looser than the call-expression path.

## Documentation coverage (non-test functions and methods)

- with a `///` doc comment: **3936** (69%)
- with only a plain `//` comment above: 30
- no description in the source: **1711**. These say _no doc comment_; their name, signature and call list are the available information.

## Call edges (non-test functions, counting call sites)

| Confidence | Call sites |
|---|---:|
| exact | 26,312 |
| by-name | 2,429 |
| ambiguous | 1,329 |
| external | 71,973 |
| macro | 4,213 |
