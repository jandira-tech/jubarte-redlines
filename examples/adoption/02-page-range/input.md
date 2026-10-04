<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Annual Maintenance Statement

Prepared for Halyard Press, Inc., 3 March 2026. This statement covers the
maintenance subscription attached to master services agreement 2026-04
and lists every intervention charged during the reporting year.

## 1. Summary of the year

The redline engine ran in production at Halyard from 14 April without an
unplanned outage. Forty-one change sets were reviewed through the
engine; thirty-eight were accepted as proposed, two were returned for
reworded clauses, and one was withdrawn by the requester. The two
returned sets were resubmitted and accepted within the same working day.

| Quarter | Change sets | Accepted | Returned | Withdrawn |
|---|---|---|---|---|
| Q1 | 9 | 8 | 1 | 0 |
| Q2 | 12 | 11 | 0 | 1 |
| Q3 | 10 | 10 | 0 | 0 |
| Q4 | 10 | 9 | 1 | 0 |

## 2. Interventions

Two interventions were chargeable under the maintenance terms. On 2 June
a header-footer regression reported by the corrections desk was traced
to a section break with an odd first-page header; the fix and its
regression test shipped the same week. On 19 September a batch of
scanned legacy agreements required a one-off font mapping for a
discontinued foundry face; the mapping table was added to the site
configuration and documented in the runbook.

Neither intervention exceeded the four-hour allowance included in each
quarter, so only the base subscription was invoiced. The quarterly
invoices of 1,900.00 USD each are reconciled in section 4.

## 3. Usage detail

The engine averaged 217 comparisons per working day, with a peak of 640
on 30 September, the day the quarterly filings were assembled. Median
turnaround from upload to reviewed redline was 41 seconds. The five most
compared counterparty forms during the year were the standard supply
agreement, the mutual NDA, the consulting services order, the software
licence, and the press distribution agreement, in that order.

The corrections desk submitted 26 of the 41 change sets; legal operations
submitted 11; the remainder came from the commercial team. No submission
was rejected for a malformed document during the year, and no submission
required manual re-entry of tracked changes.

## 4. Reconciliation and renewal

Invoices issued: four quarterly invoices of 1,900.00 USD, totaling
7,600.00 USD, all paid within terms. The subscription renews on 1 April
at the same rate if written notice is not given by 18 March. The table
below restates the year as invoiced:

| Invoice | Date | Amount (USD) | Paid |
|---|---|---|---|
| MA-2026-114 | 5 April | 1,900.00 | 30 April |
| MA-2026-241 | 5 July | 1,900.00 | 28 July |
| MA-2026-318 | 5 October | 1,900.00 | 27 October |
| MA-2026-402 | 5 January | 1,900.00 | 24 January |

We recommend renewal without change to the terms. The engine's redlines
were accepted as proposed in 38 of 41 change sets this year, and the two
layout regressions reported against Word 365 builds were resolved under
the maintenance allowance within the same week in both cases.

*The Jandira maintenance team*

## 5. Planned work for the coming year

The renewal year carries three planned improvements at no extra charge.
First, the fixture corpus gains the counterparty forms added to the
Halyard library since April, so parity checks cover the documents the
desk actually compares. Second, the font mapping table becomes a managed
artifact with its own version history, reviewed at each quarterly check-in.
Third, the comparison report gains a per-change summary line suitable for
inclusion in the corrections desk's case notes, removing a manual copy
step that today follows roughly one review in five.

Beyond the planned work, Jandira will keep the engine within one page of
Microsoft Word on documents up to the densities seen this year, will
answer regression reports within one business day, and will apply the
four-hour quarterly allowance before any chargeable time is proposed.
Any chargeable work beyond the allowance is quoted in advance and never
invoiced without written acceptance.

## 6. Contacts

For technical matters, the maintenance address reaches the on-call
engineer directly during Halyard's business hours. For invoicing and
renewal, write to the accounts desk. The runbook, the fixture inventory
and the mapping table remain available to Halyard staff at the addresses
recorded in the original handover note of 8 April.

*Signed for Jandira Technologies, LLC, 3 March 2026.*

## Appendix A. Change sets by counterparty form

The table below restates the year's 41 change sets by counterparty form
and outcome, as recorded by the engine at acceptance time. The desk can
reconcile these rows against case notes using the engine's per-review
identifiers, which are stable across reruns of the same pair.

| Form | Sets | Accepted | Returned | Withdrawn |
|---|---|---|---|---|
| Standard supply agreement | 11 | 10 | 1 | 0 |
| Mutual NDA | 9 | 9 | 0 | 0 |
| Consulting services order | 8 | 8 | 0 | 0 |
| Software licence | 7 | 6 | 0 | 1 |
| Press distribution agreement | 6 | 5 | 1 | 0 |

No other form generated more than three change sets during the year.
Returned sets were resubmitted the same day in both cases, and the
withdrawn set concerned a counterparty that consolidated its paper
mid-review rather than any defect in the redline.

## Appendix B. Statement of assurance

Jandira confirms that the figures in sections 1 and 4 were produced by
the engine's own reporting and reconciled against the invoicing ledger
on 1 March. The assurance covers the reporting year ending 29 February
and is given to support Halyard's internal audit of vendor statements.
