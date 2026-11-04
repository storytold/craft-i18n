# Khmer inherited-source evidence

Recorded 2026-11-04 for craft-i18n #122 and craft-libs #551. The proposed consumer
in #551 is unmerged. This record identifies published terms and content matches;
it records Nathan Wells/SBBIC's permission attestation and does not grant maintainer
Embeddable approval.
The [per-file Aksara record](khmer-provenance.md) contains Nathan Wells's statements.

## Findings established by the audit

### CLDR names and the historical notice

SIL first added `src/places.txt` in
[`1756a00304cff946ce404e71ef08cbc3f6984655`](https://github.com/silnrsi/khmerlbdict/commit/1756a00304cff946ce404e71ef08cbc3f6984655)
on 2015-12-21. Its 997 lines are an **exact multiset match**, including duplicates,
to the `language`, `script`, `territory` and `exemplarCity` element values in
[CLDR 28 `common/main/km.xml`](https://github.com/unicode-org/cldr/blob/660a303bd6e60f491e53b3db88977b118897cbf3/common/main/km.xml).
There are 996 nonempty name occurrences, 910 distinct nonempty names, and one
blank line corresponding to the XML's empty identity/language element.
Order differs; no spelling normalization is needed for this comparison.
This identifies matching published content, not the precise historical checkout
or extraction command used by SIL. CLDR 26 and 27 do not give that exact match.

The matching CLDR revision is `660a303bd6e60f491e53b3db88977b118897cbf3`.
Its [unicode-license.txt](https://github.com/unicode-org/cldr/blob/660a303bd6e60f491e53b3db88977b118897cbf3/unicode-license.txt)
is 2,933 bytes, SHA256
`58c22b50508c6873d2dc27390b49f94daebeb9d311464ed3eb87282cda3321f5`,
Copyright 1991–2015 Unicode, Inc. These are the historical
**Unicode-DFS-2015** terms, not a substitution of today's Unicode-3.0 notice.
The grant permits use, copying, modification and distribution, subject to retaining
the copyright/permission notice with copies and associated documentation, and
clearly identifying modifications in the data or software and its documentation.

SIL subsequently [changed the names](https://github.com/silnrsi/khmerlbdict/commit/364f03bc120aec62bdb991db21ee43ccf68ecbf5)
on 2015-12-24, splitting some strings and cleaning place spellings. Aksara combines
and changes frequencies and adds/demotes entries; the proposed Rust build converts
JSON to TSV and compresses it. **These are modified derivatives.** The manifest now
pins the historical full notice as `LICENCES/LICENSE-CLDR-28.txt`, includes a
modification statement in `LICENCES/NOTICES.txt`, and records the frequency file as
`MIT AND Unicode-DFS-2015`. The standard SPDX text is also included in archives.
The historical licence is conservatively Download-only in the checker pending
maintainer policy review. The whole pack and embedding remain gated.

### SEAlang notice retrieved directly

The project's [menu](http://sealang.net/project/list/menu.htm) links to
[license.txt](http://sealang.net/project/list/license.txt). On 2026-11-04 that file
returned the MIT notice, Copyright (c) 2010, Doug Cooper: 1,070 bytes, SHA256
`2a98708d01f8a98765501f2cc5af607c00dcef85479518876a7b5338eaa78fd6`.
Its terms match Wells's previously supplied notice; the retrieved file has extra
trailing blank lines. The terms are preserved in the manifest and pack notices.
This establishes the published frequency-list notice independently of the supplied
screenshot. It does not establish whether Aksara used SEAlang directly in addition
to the copy recorded by SIL. The evidence URL uses HTTP; its hash records the
retrieved bytes and is not a cryptographic authentication of the server.

## Input and permission confirmation

On 2026-11-04 Nathan Wells/SBBIC confirmed that Aksara used the **individual nine
`src/*.txt` files**, not SIL's generated `khmerdict.txt`. He identified the input
as `khmerlbdict-master.zip`, downloaded 2026-01-08, at SIL commit
`54abc6b87f4ffa6a0c95c09836c7c86648002273`, and reported no changes to those files.
That source revision is now identified by the contributor, rather than merely
being the snapshot inspected by this audit.

Wells also confirmed that SBBIC had permission to publish `DFD.txt`, `HC.txt`,
`TD.txt` and its additions to `names.txt` under MIT, and that SIL had permission
for its contributed lists, including `KHOV.txt`, `KHSV.txt` and `villages.txt`.
This is the named contributor's attestation, recorded alongside the upstream MIT
grant. It closes the permission questions previously left open in this record.
The record covers the redistributed word/frequency lists and SBBIC's original
contributions; it does not redistribute or grant a licence to the private books.

| SIL input | Recorded source/history | Retained terms/evidence |
|---|---|---|
| `seafreq.txt` | SEAlang; added in `1756a00`, updated in `9bcfe14` | SIL MIT; directly retrieved Doug Cooper MIT notice |
| `places.txt` | Exact CLDR 28 content match; SIL modifications in `364f03b` | SIL MIT for its changes; historical Unicode-DFS-2015 notice and modification record |
| `villages.txt` | Village/region names; Martin Hosken's additions/cleanups in `1756a00`, `8c9b613`, `fcde977` | SIL MIT; Wells confirms SIL had permission |
| `names.txt` | Martin Hosken's `6d9160e`; SBBIC additions in `7eacf04` / [PR #3](https://github.com/silnrsi/khmerlbdict/pull/3) | SIL MIT; Wells confirms permission for SIL/SBBIC contributions |
| `KHOV.txt` | Derived Bible word counts; Martin Hosken's `e063b58` | SIL MIT; Wells confirms SIL had permission |
| `KHSV.txt` | Derived Bible word counts; added with KHOV in `e063b58` | SIL MIT; same permission confirmation |
| `TD.txt` | SBBIC book frequencies, [PR #5](https://github.com/silnrsi/khmerlbdict/pull/5), `747a997` | SIL MIT; SBBIC contribution and permission confirmation |
| `HC.txt` | SBBIC book frequencies, [PR #4](https://github.com/silnrsi/khmerlbdict/pull/4), `e1b7745` | SIL MIT; SBBIC contribution and permission confirmation |
| `DFD.txt` | SBBIC book-series frequencies, [PR #2](https://github.com/silnrsi/khmerlbdict/pull/2), `2525ddf`; renamed/fixed later | SIL MIT; SBBIC contribution and permission confirmation |

No additional file-level restrictions were found in those nine files. The Unicode
notice is retained separately; the contributor's MIT confirmation does not replace it.

## Reproduced January import and subsequent changes

The committed January converter is
[`convert-dictionary.js` at e9d8e12](https://github.com/sbbic-khmer/aksara-khmer-word-processor/blob/e9d8e1241ccfde95ec858415ef02c1a4dff87871/convert-dictionary.js).
It sums raw input frequencies with equal weights, assigns word-only entries a
count of 1, drops totals below 2, and takes the top 10,000. It does not use SIL's
Makefile weights, logarithmic scaling or spelling expansions.

An independent offline reproduction using the pinned nine SIL files matches
**all 10,000 word/frequency pairs** in Aksara's committed January
[`khmer-dictionary.json`](https://github.com/sbbic-khmer/aksara-khmer-word-processor/blob/e9d8e1241ccfde95ec858415ef02c1a4dff87871/khmer-dictionary.json).
The maximum count is 161,110 and the selection cutoff is 21. This is a semantic
pair match, not a claim about JSON formatting or ordering among tied counts.

Wells reports a February merge with a word list from seven authorised SBBIC books,
followed by cleaning, AI filtering and manual changes, then July corpus-based
additions/frequency adjustments. The public
[45a54b4 commit](https://github.com/sbbic-khmer/aksara-khmer-word-processor/commit/45a54b4c57ee853b2e9e67e07b87803a7fedff94)
also describes adding and demoting dictionary entries using corpus evidence.
These later private/AI steps were not rerun by this audit; no private book text
or titles are included. This is a file-level lineage record, not a reconstruction
of each individual word's edit history.

Of the current 29,599 entries, **23,886 (80.7%)** occur literally in the SIL inputs
after removing BOM/ZWSP and separating the count fields. No coeng reordering,
spelling expansion or SIL validity filtering is applied for that overlap count.
The metric is independently reproduced. It is not an expected reproduction of
the current frequencies after the subsequent SBBIC changes.

## Reproduce the content checks

[khmer-source-audit.json](khmer-source-audit.json) pins every inspected SIL input,
the initial places file, Aksara artifact, matching CLDR XML and licence, and the
retrieved SEAlang notice by byte length and SHA256. It includes only metadata and
aggregate counts, not dictionary records or private text.

With full SIL/Aksara checkouts and the three downloaded notice/XML files:

```sh
python3 tools/audit_khmer_provenance.py \
  --sil /path/to/khmerlbdict --aksara /path/to/aksara \
  --cldr-xml /path/to/cldr-28-km.xml \
  --cldr-licence /path/to/cldr-28-unicode-license.txt \
  --sealang-licence /path/to/sealang-license.txt
```

The script does not download anything. It verifies all input hashes before
computing the match and overlaps, and fails if findings differ from the reviewed
record. The raw download URLs are recorded in the JSON, including the exact
commits; no executable source from the downloaded projects is run.

## Remaining project decision

The factual input/permission questions previously listed here are answered by
Wells's named attestation, upstream grants/notices, the exact January reproduction
and CLDR content match. No additional upstream confirmation is being requested.
Maintainers still need to review this record, approve the historical Unicode licence
class and settle the model/config ADR. The pack stays Download-only and embedding
stays disabled until that sign-off. This audit does not supply that approval.
