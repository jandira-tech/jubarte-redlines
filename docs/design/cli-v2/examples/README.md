# Proposed edit-template examples

These are concrete **design examples**, not output from an implemented `jubarte edit template` command. The proposed v2 interface has not been built or executed. The input is a disposable four-paragraph DOCX fixture, and all source and working-text hashes in the plans are real hashes of this fixture.

## Files and expected behavior

| File | Purpose |
|---|---|
| `example-source.docx` | Simple source with a payment period, a repeated anchor, double spaces and U+00A0. No revisions, comments, tables or embedded resources. |
| `template.generated.json` | Four unchanged rewrite rows, each with a verified `before_sha256`; explicit source, attribution and review policy. |
| `template.edited.json` | Two changed paragraph values and one explicit paragraph insertion. Omitted rows request no rewrite. |
| `template.exact-replace.json` | Same final text as the edited template, using literal replacements and occurrence 2 for the repeated `fee` anchor. |
| `template.composed.json` | Two disjoint replacements in p1 plus its unchanged rewrite scaffold. Demonstrates composition on one baseline. |
| `template.with-comment.json` | Same three text/structural operations as exact-replace, with an additional attached-comment request on the payment replacement. |
| `expectations.json` | Proposed optional expectation contract for edited, exact-replace and with-comment outcomes. Three executable text predicates and one deliberately unassessed commercial requirement. |
| `expected-changes.json` | Test-model expectations for the original three-effect example, including preservation requirements that the model cannot itself verify. Not a CLI receipt or a second plan dialect. |

The generated plan is valid unchanged work. If no other review or content action changes the primary document, a conforming future apply would normally return `status: ok`, `data.effect: unchanged`, and publish the exact original bytes when a primary output was requested. `--require-change` is optional. It never converts an unrealized requested edit into success.

Removing a rewrite row requests no rewrite; selected revision decisions and other explicit operations still apply. Empty paragraph text preserves the paragraph mark. Structural deletion requires `delete_paragraph`. There is no `run_operation` or `--allow-unchanged` contract.

The edited and exact-replace plans request this final logical text:

1. `Payment is due within 45 days.`
2. `Invoices must state the payment due date.`
3. `The fee is due; the charge is payable.`
4. `One  space.` — two spaces retained.
5. `Notice is due within 30 days.` — U+00A0 retained between 30 and days.

The composed example instead leaves p0 unchanged and makes p1 `The charge is due; the fee is due.` Therefore `expectations.json` intentionally fails for the generated and composed plans. It is not a universal expectation file for this directory.

## Proposed workflow

These commands illustrate the target interface; the released v0.12.0 binary does not implement it:

```sh
jubarte edit template example-source.docx -o edits.json
# Edit desired text or add typed operations.
jubarte edit apply example-source.docx --plan edits.json \
  --expect expectations.json -o revised.docx --json
```

Normal apply prepares and checks the candidate before publication. A separate `--dry-run` is optional. No output file is produced by the validation script below.

The default author is the neutral `Jubarte`. Use the author explicitly supplied by the actual task. Neither an author name, three successful operations nor a structural validation pass proves that all user requirements were implemented.

## What the validation establishes

From the package root, with Python 3.12 and `jsonschema` available:

```sh
python validate_examples.py
```

The script checks both proposed JSON Schemas, five plan examples, the expectations example, exact fixture digests, and an in-memory text model. It includes stale/bad working hashes, ID-pair disagreement, exact whitespace, overlapping literal candidate positions, disjoint same-paragraph composition, read-only observations, missing expectation targets and malformed schema inputs. It registers a timestamp format check explicitly instead of assuming an optional validator dependency enforces date-time.

`evidence/example-validation.json` records the actual result. The current run includes 21 model-case families and 19 malformed-schema cases; each family may contain several assertions. These are not 40 end-to-end DOCX tests.

The old text oracle incorrectly treated any second edit in a paragraph as an overlap. That bug in our explanatory validator is fixed: spans now resolve against one baseline and disjoint ranges compose. It was not evidence of a production Jubarte defect.

The model does **not** mutate a DOCX or establish tracked-change validity, revision settlement, comment anchors, formatting/range preservation, candidate publication, runtime support or rendering. In the with-comment example it validates and counts a declared comment request; it does not construct or inspect comment XML. Working-baseline drift is tested with an explicit string substitution, not a real accept/reject operation.

The two implementation plans specify the required production DOCX fixtures and conformance gates. The larger current engine operation set remains available through its versioned legacy contract until corresponding v2 adapters pass those gates.
