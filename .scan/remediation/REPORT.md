# Security remediation run — coverage report

- Run: security-findings remediation over the `.scan` deep-discovery output (33 dedup folders).
- Report written 2026-10-22 from: the run's per-folder outcomes (33 entries), the findings index
  `.scan/remediation/findings/all-ids.json` (123 findings, read from disk), and the run's audit
  result `{"ok":true,"problems":[]}`.

**Sources and what is verifiable where.**

- *The run's per-folder outcomes* (33 entries: folder, ok, issueNumber, issueUrl, findingIds,
  verdict, error) were supplied inline with this report's ask. Only the arithmetic fields were
  materialized to disk, at `/tmp/remediation-outcomes.json` (`folder`, `ok`, `issueNumber`,
  `findingIds` — no verdicts or errors). Every claim below attributed to a "verdict" or
  "checker's error" quotes that inline data; it is auditable only against this report's
  quotations, not against a file. Four outcomes (dedup-0011, 0013, 0014, 0026) arrived with
  empty verdict strings; the other 27 carry verdict text.
- *On disk, read for this report:* `.scan/remediation/findings/all-ids.json` (123 findings —
  the index this report is judged against), `.scan/findings.json`, the per-folder extracts in
  `.scan/remediation/findings/`, the issue artifacts in `.scan/remediation/issues/`, and the
  31 plans in `.scan/remediation/plans/` and 31 patches in `.scan/remediation/patches/`
  (counted with `ls … | wc -l`).
- *Live GitHub state,* checked this session with `gh issue view … --json` where the text says
  so: issues #1503, #1601, #1618, #1640.
- *The audit result* `{"ok":true,"problems":[]}` was supplied inline with the ask; no audit
  artifact exists under `.scan` (`find .scan -maxdepth 3 -iname '*audit*'` → nothing, exit 0),
  so §5 reports it as received, not as re-verified.

**Method (executed, not estimated).** The 33 outcomes' `findingIds` were transcribed to
`/tmp/remediation-outcomes.json` and differenced against `all-ids.json` with
`python3 /tmp/compute_coverage.py`. Two self-checks: (a) every id in the union of the outcomes'
lists exists in `all-ids.json` — the first run reported exactly **1 unknown**, and after the
dedup-0031 correction below the final run reports **0 unknowns**; (b) for every folder that
wrote one, the transcribed id set equals the on-disk `.scan/remediation/issues/*.mapped.json`
set (19/19 matches).

**One data discrepancy, resolved on disk — both arithmetics stated.** The inline outcome list
for dedup-0031 as this report received it carried `csf_06aad04a55110c49864e83c5`, an id that
exists in no findings file (it survives nowhere on disk; the corrupted string was visible only
in the ask's inline data and in the first version of the working file, which was corrected in
place before the reported numbers were computed — that correction is why self-check (a) reads
0 unknowns rather than 1). Four on-disk sources — `.scan/findings.json`,
`.scan/remediation/findings/all-ids.json`, `.scan/remediation/findings/dedup-0031.json`, and
the posted issue body `.scan/remediation/issues/dedup-0031.md` — all carry
`csf_06aad04a55110c49864e83c0` ("PDF generation can hang on a font with many retained tables"),
a one-character difference. This report counts that finding as mapped to issue #1601, giving
**79 covered / 44 unaddressed**. Taking the inline string literally instead would give **78
covered / 45 unaddressed**, because the real finding is mapped by no other row (verified: it
appears only in dedup-0031's outcome list, recomputed this session) while the corrupted string
matches no finding and would cover nothing. The disk evidence is unambiguous and four files
deep; 79/44 is the number this report stands behind.

## 1. Summary

- **33 folders** processed: **31 completed** (`ok: true`), each ending in one
  GitHub issue; **2 failed** with nothing actionable (dedup-0032, dedup-0033 — §4).
- **31 issues** stand for the run. 30 were created by it in one posting window that runs from
  #1503 (2026-10-21T18:49:44Z) to #1640 (2026-10-22T04:27:10Z), both timestamps verified live
  via `gh issue view … --json createdAt`. The 31st, dedup-0023's #1618, is a provenance outlier
  this report could not fully resolve. Verified live this session: #1618 was created by the
  account `arthrod` (human, `is_bot: false`) at 2026-10-22T01:05:53Z — inside the run's posting
  window — and its 65,516-char body carries exactly this folder's content (the 8 findingIds,
  8 `### csf_` sections with summaries, and the 6-file patch), with **0 comments**. The
  dedup-0023 verdict says the duplicate guard found the issue already open and "per the
  protocol nothing was created or commented". Whether #1618 was opened by an earlier session of
  this same run (which the content match and timestamp make most plausible) or by someone else
  cannot be distinguished from the evidence available to this report. One consequence is
  verified and material: the full finding texts prepared for #1618's comment — 14,299 bytes at
  `.scan/remediation/issues/dedup-0023.comment-01.md` — were **never posted** (live comment
  count 0), so on GitHub this issue carries compact finding summaries only; the full texts
  exist on disk alone (see the ‡ note in §2).
- **Findings coverage: 79 of 123** findings are mapped to at least
  one issue. **44 are addressed by no issue** (35 medium,
  9 low, 0 high). Of those 44,
  **33 were assigned to no folder** by the indexer (`folders: []` in `all-ids.json`) and
  **11 had folder membership** but were excluded or deferred by every folder that owned them (§3).

"Covered" means: named in an issue whose folder produced a reviewed plan and patch. **No fix has
landed in the repository.** All 27 outcome entries that carry verdict text explicitly record
that cargo/clippy/nextest (and pytest/node where relevant) were not run, being forbidden in the
preparation asks, and that no repository source was modified, committed or pushed; the four
verdict-less folders (0011, 0013, 0014, 0026) state nothing either way. Those verdicts are
quoted from the run's inline outcome data (see Sources above) — no verdict text exists on disk.

## 2. Issues opened

| Folder | Issue | URL | Mapped findingIds |
|---|---|---|---|
| dedup-0001 † | 1613 | https://github.com/jandira-tech/jubarte-redlines/issues/1613 | 25: csf_9e40f23e742aa43493a0fb92, csf_8dcaf50784fd95a7f0c2ec28, csf_ad0603057507f1d6be6e9942, csf_0e2b085a251b34412cd71a4c, csf_d46d1a3ef7bb52454cbd3c68, csf_64b247931cd1f053217fde7a, csf_f23ce2ece7c2d5de613bb58d, csf_e38139daefc493123fe13362, csf_3201e6caaef21850d61fa69c, csf_dc0d34a6ea9a964733efe547, csf_eaffa9e4af9c9c07d800ef1b, csf_a69309b0b64d1048c5a4e319, csf_6128bc397a829afd60ea9390, csf_27ca2e0d965d8b1341a32909, csf_dbf35c828f9082997b9ef8fc, csf_33c5fede5e569e4c1531e38a, csf_8510b5bc0ddac96422dfcdc6, csf_7084a83c24deb556942e2d43, csf_5a86b3f5aa767e4e64bd9e71, csf_b0aa536248aea0f44dd30d47, csf_91956fe2d99a3a2871c06916, csf_f756bbe5509c42bdcaa64921, csf_88d031991ca3e20e4aae2f24, csf_32625277093e7f286a267e2b, csf_80edacafa6b67d4864116a38 |
| dedup-0002 | 1608 | https://github.com/jandira-tech/jubarte-redlines/issues/1608 | 3: csf_9e40f23e742aa43493a0fb92, csf_8dcaf50784fd95a7f0c2ec28, csf_eaffa9e4af9c9c07d800ef1b |
| dedup-0003 † | 1626 | https://github.com/jandira-tech/jubarte-redlines/issues/1626 | 13: csf_8dcaf50784fd95a7f0c2ec28, csf_64b247931cd1f053217fde7a, csf_3201e6caaef21850d61fa69c, csf_de54caf2d47d7e03bce741e0, csf_d22452ac5ec901578d340334, csf_4de0d1e5f809171f3b8323d9, csf_550f8cf900e415a773aa959d, csf_8510b5bc0ddac96422dfcdc6, csf_4b64ae60e06cc87420036fe5, csf_7084a83c24deb556942e2d43, csf_f756bbe5509c42bdcaa64921, csf_32625277093e7f286a267e2b, csf_80edacafa6b67d4864116a38 |
| dedup-0004 † | 1631 | https://github.com/jandira-tech/jubarte-redlines/issues/1631 | 8: csf_64b247931cd1f053217fde7a, csf_9e40f23e742aa43493a0fb92, csf_4de0d1e5f809171f3b8323d9, csf_dc0d34a6ea9a964733efe547, csf_d22452ac5ec901578d340334, csf_8dcaf50784fd95a7f0c2ec28, csf_eaffa9e4af9c9c07d800ef1b, csf_de54caf2d47d7e03bce741e0 |
| dedup-0005 † | 1609 | https://github.com/jandira-tech/jubarte-redlines/issues/1609 | 12: csf_09abb95061370ae3d7d342ca, csf_32d60b431e9d66bf961b0a29, csf_de54caf2d47d7e03bce741e0, csf_4b64ae60e06cc87420036fe5, csf_3201e6caaef21850d61fa69c, csf_7084a83c24deb556942e2d43, csf_80edacafa6b67d4864116a38, csf_dc0d34a6ea9a964733efe547, csf_dbf35c828f9082997b9ef8fc, csf_91956fe2d99a3a2871c06916, csf_68acfaf376a60cf8c3d8e2d4, csf_85faece8ffd45c8ee175c288 |
| dedup-0006 † | 1630 | https://github.com/jandira-tech/jubarte-redlines/issues/1630 | 6: csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_d46d1a3ef7bb52454cbd3c68, csf_64b247931cd1f053217fde7a, csf_d22452ac5ec901578d340334, csf_49256aa9e91ee0159c917b13 |
| dedup-0007 † | 1638 | https://github.com/jandira-tech/jubarte-redlines/issues/1638 | 5: csf_8dcaf50784fd95a7f0c2ec28, csf_d22452ac5ec901578d340334, csf_68acfaf376a60cf8c3d8e2d4, csf_85faece8ffd45c8ee175c288, csf_f756bbe5509c42bdcaa64921 |
| dedup-0008 † | 1637 | https://github.com/jandira-tech/jubarte-redlines/issues/1637 | 16: csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_3201e6caaef21850d61fa69c, csf_eaffa9e4af9c9c07d800ef1b, csf_d22452ac5ec901578d340334, csf_6a40bc41bdf34a737ceac40d, csf_631ab5c4311b813e6ceb189f, csf_27ca2e0d965d8b1341a32909, csf_3ec09f0cecec11243e8a8f5e, csf_7084a83c24deb556942e2d43, csf_b0aa536248aea0f44dd30d47, csf_f756bbe5509c42bdcaa64921, csf_88d031991ca3e20e4aae2f24, csf_80edacafa6b67d4864116a38, csf_eb94605aebd8f08907c46e8c, csf_4b734e13a64fed0f0a4aaf25 |
| dedup-0009 † | 1634 | https://github.com/jandira-tech/jubarte-redlines/issues/1634 | 21: csf_0e2b085a251b34412cd71a4c, csf_09abb95061370ae3d7d342ca, csf_de54caf2d47d7e03bce741e0, csf_e317a396f5a5264629caef58, csf_cac980ab6041d4ec91b39fd1, csf_6a40bc41bdf34a737ceac40d, csf_85e0853b68d29e45897292e3, csf_d7cf4e4a5f04c299b9dd8730, csf_7bd64146a8b40f28ac601665, csf_c2ab52b1c8614c7fd5356cf8, csf_00beee9df9b14ad1518f06ce, csf_27ca2e0d965d8b1341a32909, csf_bff2c0151aeb6ffaa6f96d4a, csf_3ec09f0cecec11243e8a8f5e, csf_6054d1763a69fc51bf4be2b2, csf_4b64ae60e06cc87420036fe5, csf_b0aa536248aea0f44dd30d47, csf_f1c7e6e8877805d7436a18fa, csf_32d60b431e9d66bf961b0a29, csf_88d031991ca3e20e4aae2f24, csf_eb94605aebd8f08907c46e8c |
| dedup-0010 † | 1639 | https://github.com/jandira-tech/jubarte-redlines/issues/1639 | 11: csf_9e40f23e742aa43493a0fb92, csf_0e2b085a251b34412cd71a4c, csf_a69309b0b64d1048c5a4e319, csf_85d9c340b10d981559925227, csf_b45e3ea11925b4220c435786, csf_68acfaf376a60cf8c3d8e2d4, csf_5ad7cba70ec2176a33e66f2a, csf_dea8102957ca1306a0f7756b, csf_6a40bc41bdf34a737ceac40d, csf_c2ab52b1c8614c7fd5356cf8, csf_6128bc397a829afd60ea9390 |
| dedup-0011 | 1611 | https://github.com/jandira-tech/jubarte-redlines/issues/1611 | 8: csf_eb7afb96f6f0e2787081e030, csf_de54caf2d47d7e03bce741e0, csf_4b64ae60e06cc87420036fe5, csf_68acfaf376a60cf8c3d8e2d4, csf_85faece8ffd45c8ee175c288, csf_6a40bc41bdf34a737ceac40d, csf_3ec09f0cecec11243e8a8f5e, csf_eb94605aebd8f08907c46e8c |
| dedup-0012 † | 1632 | https://github.com/jandira-tech/jubarte-redlines/issues/1632 | 16: csf_9e40f23e742aa43493a0fb92, csf_8dcaf50784fd95a7f0c2ec28, csf_d46d1a3ef7bb52454cbd3c68, csf_dc0d34a6ea9a964733efe547, csf_68acfaf376a60cf8c3d8e2d4, csf_cac980ab6041d4ec91b39fd1, csf_c2ab52b1c8614c7fd5356cf8, csf_c1fd47c87c992981879d9be6, csf_dbf35c828f9082997b9ef8fc, csf_33c5fede5e569e4c1531e38a, csf_bff2c0151aeb6ffaa6f96d4a, csf_85faece8ffd45c8ee175c288, csf_5a86b3f5aa767e4e64bd9e71, csf_91956fe2d99a3a2871c06916, csf_f1c7e6e8877805d7436a18fa, csf_f756bbe5509c42bdcaa64921 |
| dedup-0013 | 1616 | https://github.com/jandira-tech/jubarte-redlines/issues/1616 | 4: csf_38d5438aa76b1bf2cdbb12d0, csf_5ca5660e582c260e59b28647, csf_6971fbd04aa6a684463b89f5, csf_9885a1f1ab291001eef81daa |
| dedup-0014 | 1606 | https://github.com/jandira-tech/jubarte-redlines/issues/1606 | 4: csf_0b398d69c585073b8c510a3d, csf_91c179305a37cdf9c1219fe0, csf_eb4256ff45478e77c3906750, csf_e02194eb587697016bdfe08d |
| dedup-0015 † | 1627 | https://github.com/jandira-tech/jubarte-redlines/issues/1627 | 15: csf_9e40f23e742aa43493a0fb92, csf_33c5fede5e569e4c1531e38a, csf_5a86b3f5aa767e4e64bd9e71, csf_d46d1a3ef7bb52454cbd3c68, csf_64b247931cd1f053217fde7a, csf_8510b5bc0ddac96422dfcdc6, csf_32625277093e7f286a267e2b, csf_09abb95061370ae3d7d342ca, csf_32d60b431e9d66bf961b0a29, csf_cac980ab6041d4ec91b39fd1, csf_bff2c0151aeb6ffaa6f96d4a, csf_dea8102957ca1306a0f7756b, csf_6a40bc41bdf34a737ceac40d, csf_3ec09f0cecec11243e8a8f5e, csf_eb94605aebd8f08907c46e8c |
| dedup-0016 † | 1636 | https://github.com/jandira-tech/jubarte-redlines/issues/1636 | 21: csf_9e40f23e742aa43493a0fb92, csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_64b247931cd1f053217fde7a, csf_dc0d34a6ea9a964733efe547, csf_4de0d1e5f809171f3b8323d9, csf_b45e3ea11925b4220c435786, csf_cac980ab6041d4ec91b39fd1, csf_6a40bc41bdf34a737ceac40d, csf_d7cf4e4a5f04c299b9dd8730, csf_c2ab52b1c8614c7fd5356cf8, csf_eb7afb96f6f0e2787081e030, csf_6054d1763a69fc51bf4be2b2, csf_5a86b3f5aa767e4e64bd9e71, csf_b0aa536248aea0f44dd30d47, csf_91956fe2d99a3a2871c06916, csf_f1c7e6e8877805d7436a18fa, csf_f756bbe5509c42bdcaa64921, csf_88d031991ca3e20e4aae2f24, csf_32625277093e7f286a267e2b, csf_eb94605aebd8f08907c46e8c |
| dedup-0017 † | 1612 | https://github.com/jandira-tech/jubarte-redlines/issues/1612 | 16: csf_4b734e13a64fed0f0a4aaf25, csf_631ab5c4311b813e6ceb189f, csf_32625277093e7f286a267e2b, csf_64b247931cd1f053217fde7a, csf_93c93094934792b4e7e4a94c, csf_3201e6caaef21850d61fa69c, csf_7084a83c24deb556942e2d43, csf_80edacafa6b67d4864116a38, csf_8dcaf50784fd95a7f0c2ec28, csf_f756bbe5509c42bdcaa64921, csf_09abb95061370ae3d7d342ca, csf_32d60b431e9d66bf961b0a29, csf_4b64ae60e06cc87420036fe5, csf_de54caf2d47d7e03bce741e0, csf_eb4256ff45478e77c3906750, csf_8c4451e023d8d194e7cae9a8 |
| dedup-0018 † | 1635 | https://github.com/jandira-tech/jubarte-redlines/issues/1635 | 18: csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_64b247931cd1f053217fde7a, csf_3201e6caaef21850d61fa69c, csf_dc0d34a6ea9a964733efe547, csf_eaffa9e4af9c9c07d800ef1b, csf_68acfaf376a60cf8c3d8e2d4, csf_b7534c87815cfbca97ca8543, csf_eb4256ff45478e77c3906750, csf_93c93094934792b4e7e4a94c, csf_85faece8ffd45c8ee175c288, csf_7084a83c24deb556942e2d43, csf_b0aa536248aea0f44dd30d47, csf_91956fe2d99a3a2871c06916, csf_f756bbe5509c42bdcaa64921, csf_88d031991ca3e20e4aae2f24, csf_32625277093e7f286a267e2b, csf_80edacafa6b67d4864116a38 |
| dedup-0019 † | 1625 | https://github.com/jandira-tech/jubarte-redlines/issues/1625 | 16: csf_9e40f23e742aa43493a0fb92, csf_0e2b085a251b34412cd71a4c, csf_dc0d34a6ea9a964733efe547, csf_b2aa4325c8f3aa14a940fcad, csf_85d9c340b10d981559925227, csf_b45e3ea11925b4220c435786, csf_6a40bc41bdf34a737ceac40d, csf_c2ab52b1c8614c7fd5356cf8, csf_eb4256ff45478e77c3906750, csf_58625a5255dafe3f4e5744ea, csf_5a86b3f5aa767e4e64bd9e71, csf_b0aa536248aea0f44dd30d47, csf_91956fe2d99a3a2871c06916, csf_f1c7e6e8877805d7436a18fa, csf_88d031991ca3e20e4aae2f24, csf_eb94605aebd8f08907c46e8c |
| dedup-0020 † | 1640 | https://github.com/jandira-tech/jubarte-redlines/issues/1640 | 24: csf_9e40f23e742aa43493a0fb92, csf_5a86b3f5aa767e4e64bd9e71, csf_275e734d8410a0bc01404a3c, csf_ad0603057507f1d6be6e9942, csf_0e2b085a251b34412cd71a4c, csf_b0aa536248aea0f44dd30d47, csf_88d031991ca3e20e4aae2f24, csf_d46d1a3ef7bb52454cbd3c68, csf_3201e6caaef21850d61fa69c, csf_7084a83c24deb556942e2d43, csf_80edacafa6b67d4864116a38, csf_dc0d34a6ea9a964733efe547, csf_91956fe2d99a3a2871c06916, csf_eaffa9e4af9c9c07d800ef1b, csf_68acfaf376a60cf8c3d8e2d4, csf_85faece8ffd45c8ee175c288, csf_2cc2c8f85ad501547f61a5af, csf_9178b4a4d6f60f788e9978b0, csf_cac980ab6041d4ec91b39fd1, csf_dea8102957ca1306a0f7756b, csf_6a40bc41bdf34a737ceac40d, csf_eb94605aebd8f08907c46e8c, csf_21a623ddd20cd959e828b40e, csf_8e4b0b14cb63a30d38fdbc99 |
| dedup-0021 | 1603 | https://github.com/jandira-tech/jubarte-redlines/issues/1603 | 8: csf_0e2b085a251b34412cd71a4c, csf_b0aa536248aea0f44dd30d47, csf_88d031991ca3e20e4aae2f24, csf_d46d1a3ef7bb52454cbd3c68, csf_dc0d34a6ea9a964733efe547, csf_91956fe2d99a3a2871c06916, csf_dea8102957ca1306a0f7756b, csf_b7534c87815cfbca97ca8543 |
| dedup-0022 | 1503 | https://github.com/jandira-tech/jubarte-redlines/issues/1503 | 1: csf_85e0853b68d29e45897292e3 |
| dedup-0023 ‡ | 1618 | https://github.com/jandira-tech/jubarte-redlines/issues/1618 | 8: csf_3201e6caaef21850d61fa69c, csf_eaffa9e4af9c9c07d800ef1b, csf_49256aa9e91ee0159c917b13, csf_6a40bc41bdf34a737ceac40d, csf_7084a83c24deb556942e2d43, csf_e594736f4fae83b05e87e651, csf_80edacafa6b67d4864116a38, csf_eb94605aebd8f08907c46e8c |
| dedup-0024 † | 1633 | https://github.com/jandira-tech/jubarte-redlines/issues/1633 | 13: csf_9e40f23e742aa43493a0fb92, csf_0e2b085a251b34412cd71a4c, csf_dc0d34a6ea9a964733efe547, csf_d22452ac5ec901578d340334, csf_b45e3ea11925b4220c435786, csf_6a40bc41bdf34a737ceac40d, csf_c2ab52b1c8614c7fd5356cf8, csf_5a86b3f5aa767e4e64bd9e71, csf_b0aa536248aea0f44dd30d47, csf_91956fe2d99a3a2871c06916, csf_f1c7e6e8877805d7436a18fa, csf_88d031991ca3e20e4aae2f24, csf_eb94605aebd8f08907c46e8c |
| dedup-0025 | 1622 | https://github.com/jandira-tech/jubarte-redlines/issues/1622 | 10: csf_09abb95061370ae3d7d342ca, csf_32d60b431e9d66bf961b0a29, csf_68acfaf376a60cf8c3d8e2d4, csf_2cc2c8f85ad501547f61a5af, csf_64b247931cd1f053217fde7a, csf_32625277093e7f286a267e2b, csf_e02194eb587697016bdfe08d, csf_9e40f23e742aa43493a0fb92, csf_0e2b085a251b34412cd71a4c, csf_88d031991ca3e20e4aae2f24 |
| dedup-0026 † | 1607 | https://github.com/jandira-tech/jubarte-redlines/issues/1607 | 8: csf_64b247931cd1f053217fde7a, csf_32625277093e7f286a267e2b, csf_3201e6caaef21850d61fa69c, csf_80edacafa6b67d4864116a38, csf_8dcaf50784fd95a7f0c2ec28, csf_f756bbe5509c42bdcaa64921, csf_b45e3ea11925b4220c435786, csf_c2ab52b1c8614c7fd5356cf8 |
| dedup-0027 † | 1623 | https://github.com/jandira-tech/jubarte-redlines/issues/1623 | 9: csf_9e40f23e742aa43493a0fb92, csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_64b247931cd1f053217fde7a, csf_d22452ac5ec901578d340334, csf_68acfaf376a60cf8c3d8e2d4, csf_f756bbe5509c42bdcaa64921, csf_88d031991ca3e20e4aae2f24, csf_32625277093e7f286a267e2b |
| dedup-0028 † | 1629 | https://github.com/jandira-tech/jubarte-redlines/issues/1629 | 6: csf_8dcaf50784fd95a7f0c2ec28, csf_0e2b085a251b34412cd71a4c, csf_64b247931cd1f053217fde7a, csf_3201e6caaef21850d61fa69c, csf_dc0d34a6ea9a964733efe547, csf_c2ab52b1c8614c7fd5356cf8 |
| dedup-0029 † | 1628 | https://github.com/jandira-tech/jubarte-redlines/issues/1628 | 10: csf_0e2b085a251b34412cd71a4c, csf_64b247931cd1f053217fde7a, csf_49256aa9e91ee0159c917b13, csf_cac980ab6041d4ec91b39fd1, csf_6a40bc41bdf34a737ceac40d, csf_d1ff97a577444c45ea8ba899, csf_918c9b892d0d08ee23454ad8, csf_88d031991ca3e20e4aae2f24, csf_32625277093e7f286a267e2b, csf_eb94605aebd8f08907c46e8c |
| dedup-0030 | 1615 | https://github.com/jandira-tech/jubarte-redlines/issues/1615 | 6: csf_b2aa4325c8f3aa14a940fcad, csf_64b247931cd1f053217fde7a, csf_6a40bc41bdf34a737ceac40d, csf_0e2b085a251b34412cd71a4c, csf_09abb95061370ae3d7d342ca, csf_d7cf4e4a5f04c299b9dd8730 |
| dedup-0031 | 1601 | https://github.com/jandira-tech/jubarte-redlines/issues/1601 | 7: csf_64b247931cd1f053217fde7a, csf_9e40f23e742aa43493a0fb92, csf_d46d1a3ef7bb52454cbd3c68, csf_dea8102957ca1306a0f7756b, csf_06aad04a55110c49864e83c0, csf_ee82536925c3faabcb055f4a, csf_dc0d34a6ea9a964733efe547 |

† Parts of the mandated content (full findings/plan/patch) live in comments on the issue rather
than its body, forced by GitHub's 65,536-character body cap. Source: the verdict texts in the
run's inline outcome data (quoted, not on disk); each entry matches the on-disk split artifacts
in `.scan/remediation/issues/` (`.comment-NN.md` / `.patch-comment.md` files) where they were
written:

- dedup-0001: full finding texts in 1 comment (issuecomment-6329775857)
- dedup-0003: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0004: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0005: full patch in comment 1
- dedup-0006: full finding texts in comment 1
- dedup-0007: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0008: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0009: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0010: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0012: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0015: full finding texts in comment 1 (issuecomment-6332285668)
- dedup-0016: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0017: full finding texts in comment 1 (6329716935), full plan in comment 2 (6329759127)
- dedup-0018: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0019: full finding texts in comment 1, full TDD plan in comment 2
- dedup-0020: full finding texts in comment 1, full TDD plan in comment 2, patch part 2 of 2 in comment 3
- dedup-0024: full finding texts in comment 1
- dedup-0026: patch part 2 of 2 in 1 comment (evidence: on-disk .scan/remediation/issues/dedup-0026-comment.md; verdict empty)
- dedup-0027: full finding texts in comment 1 (issuecomment-6331945888)
- dedup-0028: full finding texts in comment 1
- dedup-0029: full finding texts in comment 1, full TDD plan in comment 2

‡ **dedup-0023 (#1618) is a content gap, not a comment split.** The folder's verdict describes
a "full finding texts in 1 comment" split, and the comment was prepared on disk
(`.scan/remediation/issues/dedup-0023.comment-01.md`, 14,299 bytes, mtime Oct 21 23:13) — but
the live issue has **0 comments** (verified: `gh issue view 1618 --json comments` → `[]`), and
the verdict itself says the duplicate guard found the issue open and nothing was created or
commented. Net result, verified against the live body (65,516 chars): #1618 carries the 8
findingIds as `### csf_` sections with summaries plus the full plan and 6-file patch, but the
mandated full finding texts are **absent from GitHub entirely** — they exist only in the on-disk
file above.

Everything in the body for the remaining nine folders: dedup-0002, 0021 (findings+plan+patch in
a 54,800-char body), 0022, 0025 (explicitly 0 comments), 0030, 0031 — and dedup-0011, 0013,
0014, whose outcomes carry an **empty verdict**: `ok: true` with issue number, URL and findingIds
recorded, but no checker's verdict text, so no double-check evidence and no comment-split
information exists for them (dedup-0026 is also verdict-less; its comment footnote above rests
on the on-disk artifact).

Numbering: #1503 (dedup-0022) is the run's low outlier, created 2026-10-21T18:49:44Z — about
two hours before the main batch begins at #1601 (21:01:52Z); issues 1504–1600 pre-date this run
and are unrelated repository history.

## 3. Findings not addressed

Every findingId in `all-ids.json` absent from the union of the outcomes' `findingIds` — computed,
not estimated (44 of 123).

### 3a. Had folder membership, but no owning folder's issue mapped them (11)

| FindingId | Severity | Title | Folder membership (all-ids.json) |
|---|---|---|---|
| `csf_a9b4e9d87eb567391149dad1` | medium | Large page coordinates make border rendering loop without progress | dedup-0002 |
| `csf_3b8aeff5efd1bfbd5ccfd3dd` | medium | Scrubbing can silently preserve identifying UTF-16 core and app properties | dedup-0005 |
| `csf_d85bbb63ac7a034dda69d78b` | medium | Listing many comments over one large range multiplies retained anchor text | dedup-0009 |
| `csf_8510417e640fe194edfa6571` | medium | Streaming XML hash repeatedly rescans growing escaped opening tags | dedup-0011 |
| `csf_30d7378aa76256da61dfe8fc` | medium | Nested semantic fields multiply retained instructions and hash work | dedup-0011 |
| `csf_08670176ab69a0d459ba1059` | medium | Reference metadata stamping builds oversized ZIP indexes before refusal | dedup-0017 |
| `csf_6aa9a8626de9bc707c34e4d3` | medium | Repeated compressed-image references can exhaust renderer memory | dedup-0018, dedup-0024, dedup-0029 |
| `csf_5b24e6910eeca1b1d4367847` | medium | Document font names trigger quadratic persistent index rewrites | dedup-0021 |
| `csf_d7c1eb34fffbe325bc2b0af4` | medium | Windows Markdown image checks can contact an external share before rejecting it | dedup-0024 |
| `csf_50c9eabc4578186352265845` | low | Rejecting malformed ZIP headers can allocate gigabytes of diagnostic text | dedup-0025 |
| `csf_1bcd308b4a2806997600d807` | low | Python CLI output/plan and npm direct-output symlinks can overwrite unrelated files | dedup-0004, dedup-0010, dedup-0013, dedup-0015 |

Grounds, where the run's inline verdict texts state one (quoted — none of this is on disk):
csf_3b8aeff5 stayed out of scope in dedup-0005 ("the other
4 … carry none and stayed out of scope"); csf_d85bbb63 is one of dedup-0009's 11
documented-not-fixed exclusions (comment anchor); csf_08670176 was excluded by dedup-0017
(carries no dedup0017 evidence); csf_6aa9a862 was "deliberately not mapped" by dedup-0018 and
excluded by dedup-0024 as "an unfixed gap" the plan documents; csf_d7c1eb34 was "excluded as
untouched" by dedup-0024 (Windows Markdown UNC); csf_50c9eabc is among dedup-0025's six
documented-unpatched (ZIP diagnostics). For csf_a9b4e9d8 (dedup-0002), csf_5b24e691
(dedup-0021), csf_8510417e and csf_30d7378a (dedup-0011, empty verdict), and csf_1bcd308b (a
duplicate-title twin of the covered csf_b45e3ea1, in four folders' extracts but never mapped by
any), no verdict records a rationale.

Auditability caveat, checked this session: a `grep -rl` of all 11 ids over
`.scan/remediation/plans/`, `.scan/remediation/patches/` and `.scan/remediation/issues/` finds
**10 of the 11 in no on-disk artifact at all** — only csf_6aa9a862 appears, and only in
`.scan/remediation/issues/dedup-0024.md`, a superseded single-file draft superseded by the
`.body.md`/`.comment-01.md` split that is live as #1633. The exclusion grounds above therefore
rest solely on the run's own outcome record and cannot be re-verified from this repository.

### 3b. Assigned to no folder by the indexer (33)

| FindingId | Severity | Title |
|---|---|---|
| `csf_8dc0178c218380a4337cf074` | medium | Unbounded HTML colspan exhausts memory during DOCX rendering |
| `csf_2ece2f7902f77a603a8c3e7d` | medium | DOCX table spans drive attacker-sized column allocations |
| `csf_656f2c09e37719ab674c7d01` | medium | HTML altChunk colspan expands into unbounded generated XML |
| `csf_64287538a7a91a3131e41cf3` | medium | Unbounded numbering levels drive billions of renderer operations |
| `csf_ab1b5981b2fe3523acda9227` | medium | Chicago numbering expands one counter into a multi-gigabyte label |
| `csf_8e223ee6b4dac7611a7a1666` | medium | Non-XML auxiliary filenames bypass admission before recursive parsing |
| `csf_cfe16e57b9ef3db1480af379` | medium | Decoded relationship ampersands trigger quadratic target resolution |
| `csf_7d0ce29936a1160934d4fc7c` | medium | Identical duplicate relationship repair performs quadratic scans |
| `csf_b6323ea4d1b9deec7fd1691b` | medium | Intern-pool saturation duplicates long namespace URIs per element |
| `csf_5c8dea5553c7fc1081bb4404` | medium | Markdown XML tree duplicates inherited namespace maps for each child declaration |
| `csf_6ac92b590b334d3bf00acb96` | medium | Markdown reference DOCX reaches recursive XML parsing before admission |
| `csf_0529dc7f423199b735cc79c0` | medium | Tracked-edit audit parses permission-bearing XML before admission |
| `csf_c68801f07e53c21ee989186f` | low | Python CLI output writes follow substituted symlinks |
| `csf_17d1c1cbd835bb262cc60f1b` | low | npm CLI output writes follow substituted symlinks |
| `csf_d681729bf32d41763310113e` | low | Python plan recording follows predictable temporary-file symlinks |
| `csf_a2fc12fcad36bf0a916c1879` | low | npm plan recording follows predictable temporary-file symlinks |
| `csf_d633eef7259aae77e07e54f3` | low | A shared font-index parent permits symlink-directed cache staging writes |
| `csf_a10f0e5f40cc0a10cb3e3a1a` | medium | Explicit Markdown revision IDs trigger quadratic run splitting |
| `csf_996bada33e4ee3093dba3ed9` | medium | Rendering a DOCX with renamed auxiliary XML can exhaust the call stack |
| `csf_9bec1c0d5ba7855608bbae36` | medium | Converting Markdown with an untrusted reference DOCX can exhaust the call stack |
| `csf_c55dc55a00e0fa344843906e` | medium | Redacting text can leave recoverable copies in UTF-16 XML |
| `csf_d84eab4d13fbce226cc028c6` | medium | Converting cyclic Markdown drawing payloads can terminate the host process |
| `csf_15a49e54554d7e43955ad841` | medium | Rendering a DOCX with a large table span can exhaust host memory |
| `csf_f2b8de5e6202ea345a0e0c02` | medium | Rendering embedded HTML tables can expand a tiny colspan into billions of XML elements |
| `csf_c1fe8475183ebde207aa0c59` | medium | Rendering nonstandard auxiliary XML parts bypasses admission depth limits |
| `csf_3339fb9302ddc111d50992db` | medium | Rendering Chicago numbering can allocate gigabytes for one short counter |
| `csf_97bbc446dda426be0cba1425` | medium | Document numbering values allocate multi-gigabyte Chicago labels |
| `csf_e3e47288f289bb478cceac76` | medium | Attacker-controlled list levels cause billions of iterations and counter allocations |
| `csf_40b66860adb64bbff7afac76` | medium | HTML altChunk colspan causes unbounded generated XML expansion |
| `csf_5722c422d8b3b400072d52fe` | low | Python and npm plan recording follows precreated staging symlinks |
| `csf_a3d19de450bebd004df5f6d1` | low | Recording a plan in a shared directory can overwrite a symlink target |
| `csf_cb06d76f5ddd08a0273f8203` | medium | Converting a small HTML altChunk can exhaust memory |
| `csf_8b6d07b4362cb6555001c97b` | medium | Rendering a small DOCX table can allocate billions of columns |

These are the findings with `folders: []` in `all-ids.json`. Per the dedup-0032/0033 checkers'
analysis, 17 of the 33 carry workerId 9d5d6f09-b308-4879-94e1-e57c3f808ade — the worker whose
two reducer rounds (0032 and its byte-identical retry 0033) wrote no output before the scan
sealed; the remaining 16 were never attached to any round's output by the membership rule
(union of `provenance.sourceFindingIds` prefixes and `dedupNNNN` evidence refs). The dedup-0033
checker further judged, by title/location comparison only, that ~12 of the 17 worker-9d5d6f09
findings are substantively subsumed by findings other folders did patch (table gridSpan, HTML
colspan, Chicago numbering, list levels, auxiliary-XML admission, plan-recording symlinks,
font-index staging, duplicate-relationship repair), and ~5 appear novel or partially novel:
csf_b6323ea4 (xmllinq intern-pool saturation), csf_5c8dea55 (markdown inherited namespace-map
duplication), csf_a10f0e5f (split_for_ids quadratic run splitting), csf_cfe16e57 (relationship-
target double-decode; its quadratic half matches covered findings), csf_0529dc7f (audit_tracked
parsing before admission). That is the checker's assessment, cited as such; subsumption was not
verified against source in this report.

## 4. Folders without an issue

### dedup-0032 — `ok: false`, no issue, no findings mapped

Checker's error (verbatim from the outcome):

> dedup-0032 documents nothing actionable: its reducer round never produced output. Evidence (all checked this session): (1) the ask's named inputs are absent — .scan/artifacts/deep_discovery/dedup/dedup-0032/output/ is empty (ls -laR): no result.json, no threatmodel.md; only prompt.md exists, claiming worker 9d5d6f09-b308-4879-94e1-e57c3f808ade. (2) .scan/scan-manifest.json reports scan status="failed", sealedAt=2026-10-20T22:24:59.082312Z, last recorded reducerConfiguration.reducerLabel="dedup-0031"; sibling rounds 0030/0031 each wrote ~30MB result.json + ~500KB threatmodel.md, and the dedup-0033 retry (same claimed worker 9d5d6f09) also wrote nothing before the scan sealed. (3) Of the 123 findings in .scan/findings.json, 0 have attackPath.corroboratingPaths[].evidenceRefs starting with "dedup0032" (this round's own contributions) and 0 have provenance.sourceFindingIds starting with the claimed worker UUID — the exact two branches that map findings to folders (the worker branch gave dedup-0030 and dedup-0031 their 10 findings each). (4) The workflow's own membership map .scan/remediation/findings/dedup-0032.json assigns 0 findings to this folder. (5) The 17 findings whose retained source bodies carry workerId 9d5d6f09 are among 33 findings unmapped to ANY folder — inherited base-aggregate material no dedup round synthesized over; adopting them would exceed this folder's mandate. No patch or plan file was written, per the instruction not to invent one.

### dedup-0033 — `ok: false`, no issue, no findings mapped

Checker's error (verbatim from the outcome):

> The folder documents nothing actionable for a patch preparer. Verified this session: (1) dedup-0033/output/ is empty — no result.json, no threatmodel.md, no checkpoints/; the only file in the folder is prompt.md (find over .scan/artifacts/deep_discovery/dedup/dedup-0033). The '~5MB result.json / ~160KB threatmodel.md' in my instructions is a static template string in the workflow draft (Security-findings-remediation.dwf.ts:173), not this folder's reality; the reducer round never saved an aggregate (dedup-0032 is a byte-identical assignment — same claimedWorkerId 9d5d6f09-b308-4879-94e1-e57c3f808ade, prompts differ only in the reducerLabel — and its output dir is empty too; the host apparently re-ran the same reduction after the first attempt recorded nothing). (2) The workflow's finding map assigns this folder zero findings: .scan/remediation/findings/dedup-0033.json is {folder: dedup-0033, findings: []}, and I re-derived both membership signals myself over .scan/findings.json — grep -c 'dedup0033' returns 0 (also 0 in exports/results.sarif), and no provenance.sourceFindingIds entry of any of the 123 findings starts with the claimedWorkerId. The UUID appears only as provenance.workerId, a field the membership rule (build_finding_map.py: union of sourceFindingIds-prefix and dedupNNNN evidenceRefs) deliberately does not use. (3) Coverage consequence for the run owner, stated plainly: the 17 findings that carry provenance.workerId = 9d5d6f09-b308-4879-94e1-e57c3f808ade land in NO folder under that rule (verified against all-ids.json: 0 of the 17 appear in any folder list), so no preparer in this workflow owns them; the coverage reporter's notCovered list is the designed channel for exactly this class (indexer prompt: 'anything that does not [land in at least one folder file] is counted in all-ids.json only'). By title/location comparison (not source reading), ~12 of the 17 are substantively subsumed by earlier-round-owned findings (table gridSpan allocation, HTML colspan expansion, Chicago numbering, list levels, auxiliary-XML admission bypass, plan-recording symlink staging, font-index staging, duplicate-relationship repair); ~5 appear novel or partially novel — csf_b6323ea4 xmllinq intern-pool saturation (src/xmllinq/mod.rs, shared with 0 other findings), csf_5c8dea55 markdown inherited namespace-map duplication (src/markdown/from_docx/ooxml.rs), csf_a10f0e5f split_for_ids quadratic run splitting, csf_cfe16e57 relationship-target double-decode (its quadratic unescape_xml_text half matches the earlier 'entity decoding repeatedly scans the input suffix' findings), csf_0529dc7f audit_tracked parsing before admission (src/validate.rs, shared with 0 other findings). I did not patch these: they are outside this folder's assignment under the workflow's own rule, claiming them would duplicate the byte-identical dedup-0032 mandate (two patches on the same sinks and two GitHub issues for the same findings — the defect the issue auditor is explicitly told to flag), and the ask forbids inventing a patch for a folder that documents nothing. No instruction conflict requiring escalation was found: the ok=false path is satisfiable and accurate.

Both folders document nothing actionable: their reducer rounds never wrote output, the
workflow's own finding map assigns each of them zero findings, and both checkers explicitly
declined to invent a patch or claim the 17 unmapped worker-9d5d6f09 findings, which would have
duplicated the byte-identical sibling mandate. The consequence — those findings owning no
folder — is recorded in §3b.

## 5. Audit problems

None reported. The audit result supplied inline with this report's ask is
`{"ok": true, "problems": []}`. It is reported here as received: no audit artifact exists
anywhere under `.scan` (checked this session: `find .scan -maxdepth 3 -iname '*audit*'` → no
output, exit 0), so the passing gate cannot be re-verified from this repository.

## 6. Conclusion

**44 of the 123 findings that needed a fix were not addressed by any issue in this run**
(35 medium, 9 low, 0 high):

- **33 never became actionable** — the indexer's membership rule attached them to no
  folder, so no preparer owned them. The proximate recorded cause for 17 of them is the two
  failed reducer rounds (dedup-0032 and its byte-identical retry dedup-0033), whose claimed
  worker 9d5d6f09 wrote no aggregate before the scan sealed.
- **11 were owned and then dropped** — folders listed them in their extracts but
  excluded or deferred them: compressed-image references (#1635/#1633 defer as an unfixed gap),
  ZIP index stamping (#1612 excluded), UTF-16 core/app property scrub (#1609 out of scope),
  comment-anchor text multiplication (#1634 documented-not-fixed), ZIP diagnostic text (#1622
  unpatched), Windows Markdown UNC share contact (#1633 untouched), border-rendering loop
  (#1608 unmapped), font-name persistent-index rewrites (#1603 unmapped), streaming XML hash
  rescan and nested semantic fields (#1611, empty verdict), and one duplicate-title symlink
  twin never mapped by any of its four owning folders.

The other 79 findings are covered by 31 issues that each carry a reviewed plan and patch. Per
the verdict texts in the run's inline outcome data (quoted, not on disk), those patches were
double-checked against source (`git apply --check`, rustfmt/clippy-driver probes, hand-traced
semantics) — but the cargo/test verification cycle was never executed and the repository working
tree was never modified. Where the committer finds everything, counted this session: 31 plans in
`.scan/remediation/plans/`, 31 patches in `.scan/remediation/patches/` (one pair per ok folder),
plus the same content embedded in the 31 issue bodies and their comments — except #1618's full
finding texts, which were never posted and live only at
`.scan/remediation/issues/dedup-0023.comment-01.md`. Landing the fixes is the committers' next
step; the unaddressed 44 are the residual backlog.
