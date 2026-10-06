#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")"
JUBARTE="${JUBARTE:-jubarte}"

PROFILE=file:///tmp/lo_adopt_08

# Inputs: two Markdown versions of one letter, built by the jubarte Markdown
# writer (this task needs two documents, so the default input path applies
# to each side).
"$JUBARTE" convert original.md -o original.docx --force
"$JUBARTE" convert revised.md -o revised.docx --force

# The redline under test: Word tracked changes, author Ann.
"$JUBARTE" original.docx revised.docx -o redline.docx --author Ann --force
"$JUBARTE" changes redline.docx --json > changes_jubarte.json

# --- Side 1: LibreOffice accept-all ---------------------------------------
if command -v soffice >/dev/null; then
  # The documented runner is the Python-UNO bridge. Probe the two
  # interpreters named on the adoption pages and record the results.
  : > uno_probe.log
  UNO_PY=""
  for py in /opt/homebrew/bin/python3 /usr/bin/python3 \
            /Applications/LibreOffice.app/Contents/Resources/python; do
    if [ -x "$py" ]; then
      if "$py" -c 'import uno' >> uno_probe.log 2>&1; then
        echo "uno: OK with $py" >> uno_probe.log
        UNO_PY="$py"
        break
      else
        echo "uno: FAILED with $py (exit $?)" >> uno_probe.log
      fi
    else
      echo "uno: interpreter not present: $py" >> uno_probe.log
    fi
  done

  if [ -n "$UNO_PY" ]; then
    "$UNO_PY" libreoffice_accept.py redline.docx accepted_libreoffice.docx \
      || echo "uno path exit=$?" >> uno_probe.log
  else
    # No runnable uno interpreter: dispatch the same .uno:AcceptAllTrackedChanges
    # from a Basic macro installed in this folder's own LibreOffice profile.
    rm -rf /tmp/lo_adopt_08
    timeout 180 soffice -env:UserInstallation="$PROFILE" --headless --norestore \
      --terminate_after_init >/dev/null 2>&1 || true
    mkdir -p /tmp/lo_adopt_08/user/basic/Standard
    cat > /tmp/lo_adopt_08/user/basic/Standard/Module1.xba <<'XBA'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE script:module PUBLIC "-//OpenOffice.org//DTD OfficeDocument 1.0//EN" "module.dtd">
<script:module xmlns:script="http://openoffice.org/2000/script" script:name="Module1" script:language="StarBasic">Sub AcceptAllTrackedChanges(sIn As String, sOut As String)
  Dim oDesktop As Object, oDoc As Object, oFrame As Object, oDisp As Object
  Dim oLoad(0) As New com.sun.star.beans.PropertyValue
  Dim oStore(0) As New com.sun.star.beans.PropertyValue
  oDesktop = createUnoService(&quot;com.sun.star.frame.Desktop&quot;)
  oLoad(0).Name = &quot;Hidden&quot; : oLoad(0).Value = True
  oDoc = oDesktop.loadComponentFromURL(ConvertToURL(sIn), &quot;_blank&quot;, 0, oLoad())
  oFrame = oDoc.CurrentController.Frame
  oDisp = createUnoService(&quot;com.sun.star.frame.DispatchHelper&quot;)
  oDisp.executeDispatch(oFrame, &quot;.uno:AcceptAllTrackedChanges&quot;, &quot;&quot;, 0, Array())
  oStore(0).Name = &quot;FilterName&quot; : oStore(0).Value = &quot;MS Word 2007 XML&quot;
  oDoc.storeToURL(ConvertToURL(sOut), oStore())
  oDoc.close(False)
End Sub</script:module>
XBA
    IN="$(pwd)/redline.docx"
    OUT="$(pwd)/accepted_libreoffice.docx"
    timeout 180 soffice -env:UserInstallation="$PROFILE" --headless --norestore \
      "macro:///Standard.Module1.AcceptAllTrackedChanges(\"$IN\",\"$OUT\")" \
      > basic_macro.log 2>&1 || echo "macro run exit=$?" >> basic_macro.log
    if [ ! -f accepted_libreoffice.docx ]; then
      echo "LibreOffice macro produced no output" >&2
      exit 1
    fi
  fi
else
  echo "skip: soffice not installed"
fi

# --- Side 2: jubarte accept / reject ---------------------------------------
"$JUBARTE" accept redline.docx -o accepted_jubarte.docx --force
"$JUBARTE" reject redline.docx -o rejected_jubarte.docx --force

# Text comparison of the two accepted files, and of the rejected file
# against the original.
"$JUBARTE" text accepted_jubarte.docx > accept_text_jubarte.txt
if [ -f accepted_libreoffice.docx ]; then
  "$JUBARTE" text accepted_libreoffice.docx > accept_text_libreoffice.txt
  diff accept_text_jubarte.txt accept_text_libreoffice.txt \
    > accept_text.diff || true
fi
"$JUBARTE" text rejected_jubarte.docx > reject_text_jubarte.txt
"$JUBARTE" text original.docx > text_original.txt
diff reject_text_jubarte.txt text_original.txt > reject_text.diff || true

# No tracked change may survive on either side.
"$JUBARTE" revisions accepted_jubarte.docx > revisions_accepted_jubarte.log 2>&1 || true
if [ -f accepted_libreoffice.docx ]; then
  "$JUBARTE" revisions accepted_libreoffice.docx > revisions_accepted_libreoffice.log 2>&1 || true
fi

# Page 1 of each accepted file, rendered by jubarte (72 dpi).
"$JUBARTE" convert accepted_jubarte.docx --png --dpi 72 --force >/dev/null
mv accepted_jubarte-page-01.png accept_page_1_jubarte.png 2>/dev/null \
  || mv accepted_jubarte-page-1.png accept_page_1_jubarte.png
if [ -f accepted_libreoffice.docx ]; then
  "$JUBARTE" convert accepted_libreoffice.docx --png --dpi 72 --force >/dev/null
  mv accepted_libreoffice-page-01.png accept_page_1_libreoffice.png 2>/dev/null \
    || mv accepted_libreoffice-page-1.png accept_page_1_libreoffice.png
fi

{
  echo "jubarte: $("$JUBARTE" --version)"
  if command -v soffice >/dev/null; then echo "soffice: $(soffice --version 2>/dev/null | head -1)"; else echo "soffice: not installed"; fi
} > versions.txt

echo "done"
