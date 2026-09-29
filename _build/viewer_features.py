"""Viewer features on top of the engine-comparison template (engine_compare.HTML_TEMPLATE):

* a drop-down that switches between the DOCX to PDF comparison (``index.html``) and the
  redline comparison (``redlines/index.html``);
* case filters in the side panel: group, reference page count, one engine's score at most
  X on one metric, the engine's page count differing from Word's; a count of the cases
  shown, a random-case button (key ``r``) and a clear button;
* a random case when the page opens without a ``#group/case`` link, instead of the first.

``patch(html, mode)`` works on the template and on an already built page alike (the data
sits between the anchors, never inside them); it is idempotent. ``build_site.py`` applies
it through ``patched_template(note, mode)``; run this file to patch the published pages in
place without rebuilding them:

    python3 _build/viewer_features.py            # index.html and redlines/index.html
"""

from __future__ import annotations

import sys
from pathlib import Path

MARKER = "/* viewer-features v1 */"
MODES = {
    "convert": ("Convert DOCX → PDF", {"convert": "./", "redline": "redlines/"}),
    "redline": ("Compare (redline) two documents", {"convert": "../", "redline": "./"}),
}

CSS = MARKER + """
#bar select#mode { font-weight:600; }
#sidehead { position:sticky; top:0; z-index:2; background:var(--panel); border-bottom:1px solid var(--border); }
#sidehead #filter { position:static; }
#fx { display:grid; grid-template-columns:auto 1fr; gap:4px 6px; padding:6px 8px; font-size:12px; align-items:center; }
#fx label { color:var(--muted); }
#fx select, #fx input, #fx button { background:#333; color:var(--fg); border:1px solid #555; border-radius:3px; padding:1px 4px; font:inherit; width:auto; position:static; }
#fx .row { display:flex; gap:4px; align-items:center; flex-wrap:wrap; }
#fx input[type=number] { width:4.5em; }
#fx .foot { grid-column:1/3; display:flex; gap:6px; align-items:center; flex-wrap:wrap; padding-right:4px; }
#fx .foot #fxn { color:var(--muted); margin-right:auto; }
#fx button:hover { background:#444; }
#list .case { scroll-margin-top:var(--headh, 0px); }
"""

FX_HTML = """<div id="sidehead"><input id="filter" placeholder="filter cases (name, group)…"><div id="fx">
  <label for="fxGroup">group</label><select id="fxGroup"><option value="">all groups</option></select>
  <label for="fxPages">pages</label><select id="fxPages"><option value="">any</option><option value="1-1">1</option><option value="2-3">2–3</option><option value="4-10">4–10</option><option value="11-">11+</option></select>
  <label for="fxEng">score</label><span class="row"><select id="fxEng"></select><select id="fxMetric"></select> ≤ <input type="number" id="fxMax" min="0" max="100" step="1" placeholder="any"></span>
  <label for="fxDiffer">pages</label><span class="row"><label><input type="checkbox" id="fxDiffer"> differ from Word</label></span>
  <span class="foot"><span id="fxn"></span><button id="fxRand" title="open a random case of this list (r)">random</button><button id="fxClear" title="clear every filter">clear</button></span>
</div></div>"""

FX_JS = r"""
// Case filters (viewer-features): group, reference page count, one engine's score, page count.
function refPages(c) { return (c.page_counts || {}).reference ?? (c.pages.reference || []).length; }
function passesFx(c) {
  const fx = state.fx;
  if (fx.group && c.group !== fx.group) return false;
  if (fx.pages) {
    const [lo, hi] = fx.pages.split('-').map(x => x === '' ? Infinity : +x);
    const n = refPages(c); if (n < lo || n > hi) return false;
  }
  if (fx.max !== '' && fx.max != null) {
    const v = c.scores[fx.eng]?.[fx.metric];
    if (v == null || v > +fx.max) return false;
  }
  if (fx.differ) {
    const n = (c.page_counts || {})[fx.eng];
    if (n == null || n === refPages(c)) return false;
  }
  return true;
}
// Scroll the selected case to the middle of the list area below the sticky filters.
function showSel() {
  const side = $('#side'), sel = document.querySelector('#side .sel'), head = $('#sidehead').offsetHeight;
  $('#list').style.setProperty('--headh', head + 'px');
  if (!sel) return;
  const room = side.clientHeight - head;
  side.scrollTop += sel.getBoundingClientRect().top - side.getBoundingClientRect().top - head - Math.max(0, (room - sel.offsetHeight) / 2);
}
function pickRandom() {
  const vis = visibleCases();
  if (vis.length) state.sel = vis[Math.floor(Math.random() * vis.length)][1];
  state.view = 'viewer'; state.shown = PAGE_STEP;
}
function fxChanged() {
  const vis = visibleCases().map(([, i]) => i);
  if (vis.length && !vis.includes(state.sel)) { state.sel = vis[0]; state.shown = PAGE_STEP; }
  render();
}
function setupFx() {
  const fx = state.fx, groups = [...new Set(DATA.map(c => c.group))].sort();
  for (const g of groups) { const o = document.createElement('option'); o.value = g; o.textContent = g; $('#fxGroup').appendChild(o); }
  for (const [k, label] of ENGINES) if (k !== 'reference') { const o = document.createElement('option'); o.value = k; o.textContent = label; $('#fxEng').appendChild(o); }
  for (const m of METRICS) { const o = document.createElement('option'); o.value = m; o.textContent = METRIC_LABEL[m]; o.title = METRIC_INFO[m]; $('#fxMetric').appendChild(o); }
  if (!groups.includes(fx.group)) fx.group = '';
  if (!ENGINES.some(([k]) => k === fx.eng && k !== 'reference')) fx.eng = ENGINES.find(([k]) => k !== 'reference')?.[0] ?? '';
  if (!METRICS.includes(fx.metric)) fx.metric = METRICS[0];
  $('#fxGroup').value = fx.group; $('#fxPages').value = fx.pages; $('#fxEng').value = fx.eng;
  $('#fxMetric').value = fx.metric; $('#fxMax').value = fx.max; $('#fxDiffer').checked = fx.differ;
  $('#fxGroup').onchange = e => { fx.group = e.target.value; fxChanged(); };
  $('#fxPages').onchange = e => { fx.pages = e.target.value; fxChanged(); };
  $('#fxEng').onchange = e => { fx.eng = e.target.value; fxChanged(); };
  $('#fxMetric').onchange = e => { fx.metric = e.target.value; fxChanged(); };
  $('#fxMax').oninput = e => { fx.max = e.target.value; fxChanged(); };
  $('#fxDiffer').onchange = e => { fx.differ = e.target.checked; fxChanged(); };
  $('#fxRand').onclick = () => { pickRandom(); render(); showSel(); };
  $('#fxClear').onclick = () => {
    Object.assign(fx, { group: '', pages: '', max: '', differ: false }); state.filter = '';
    $('#filter').value = ''; $('#fxGroup').value = ''; $('#fxPages').value = ''; $('#fxMax').value = ''; $('#fxDiffer').checked = false;
    fxChanged();
  };
  $('#mode').onchange = e => { location.href = e.target.value; };
}
"""


def mode_select(mode: str) -> str:
    """The drop-down: this page selected, the other one a link (relative to this page)."""
    here = MODES[mode][1][mode]
    options = "".join(
        f'<option value="{MODES[mode][1][m]}"{" selected" if MODES[mode][1][m] == here else ""}>{label}</option>'
        for m, (label, _) in MODES.items()
    )
    return f'<span class="grp"><select id="mode" autocomplete="off" title="what to compare">{options}</select></span>'


def swaps(mode: str) -> list[tuple[str, str]]:
    """(anchor, replacement) pairs; every anchor must occur exactly once."""
    return [
        ("</style>", CSS + "</style>"),
        ('<div id="bar">\n', '<div id="bar">\n  ' + mode_select(mode) + "\n"),
        ('<input id="filter" placeholder="filter cases (name, group)…">', FX_HTML),  # FX_HTML wraps the same input
        ("for (const [k] of ENGINES) if (state.on[k] == null) state.on[k] = true;",
         "for (const [k] of ENGINES) if (state.on[k] == null) state.on[k] = true;\n"
         "state.fx = Object.assign({ group: '', pages: '', eng: '', metric: '', max: '', differ: false }, state.fx || {});"),
        ("applyHash();\nwindow.onhashchange", "const linked = applyHash();\nwindow.onhashchange"),
        ("  return DATA.map((c,i) => [c,i]).filter(([c]) => !f || (c.case + ' ' + c.group).toLowerCase().includes(f));\n}\n",
         "  return DATA.map((c,i) => [c,i]).filter(([c]) => (!f || (c.case + ' ' + c.group).toLowerCase().includes(f)) && passesFx(c));\n}\n"
         + FX_JS),
        ("  const list = $('#list'); list.innerHTML = '';",
         "  const list = $('#list'); list.innerHTML = '';\n"
         "  $('#fxn').textContent = `${visibleCases().length} of ${DATA.length} cases`;"),
        ("$('#filter').oninput = e => { state.filter = e.target.value; renderList(); };",
         "$('#filter').oninput = e => { state.filter = e.target.value; renderList(); };\n"
         "setupFx();\nif (!linked) pickRandom();"),
        ("if (e.target.tagName === 'INPUT' && e.target.type === 'text') return;",
         "if (e.target.tagName === 'SELECT' || (e.target.tagName === 'INPUT' && ['text', 'number'].includes(e.target.type))) return;"),
        ("  else if (e.key === 'o') state.ovl = !state.ovl;",
         "  else if (e.key === 'r') pickRandom();\n  else if (e.key === 'o') state.ovl = !state.ovl;"),
        ("<kbd>o</kbd> overlay", "<kbd>r</kbd> random &nbsp;<kbd>o</kbd> overlay"),
        # The opening (random) case is somewhere down the list: bring it into view.
        ("\nrender();\n</script>",
         "\nrender();\nshowSel();\n</script>"),
    ]


def patch(html: str, mode: str) -> str:
    if MARKER in html:
        return html
    for old, new in swaps(mode):
        if html.count(old) != 1:
            raise SystemExit(f"viewer anchor not found once ({html.count(old)}x): {old[:70]!r}")
        html = html.replace(old, new)
    return html


def main(argv: list[str]) -> int:
    site = Path(__file__).resolve().parent.parent
    for rel, mode in (("index.html", "convert"), ("redlines/index.html", "redline")):
        path = site / rel
        before = path.read_text()
        after = patch(before, mode)
        if after != before:
            path.write_text(after)
        print(f"{rel}: {'patched' if after != before else 'already patched'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
