#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Accept every tracked change with LibreOffice: ``.uno:AcceptAllTrackedChanges``.

This is the LibreOffice command a soffice-driven skill dispatches (Anthropic's
``scripts/accept_changes.py`` dispatches it from a Basic macro). Here it runs
through LibreOffice's own Python-UNO bridge on a hidden document in a
throwaway profile, then stores the result as Word 2007-365 ``.docx``.

    python3 libreoffice_accept.py in.docx out.docx

Needs ``soffice`` and the ``uno`` module LibreOffice ships (Debian/Ubuntu:
``python3-uno``). Uses the system Python, not a virtualenv.
"""

from __future__ import annotations

import contextlib
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import uno
from com.sun.star.beans import PropertyValue
from com.sun.star.connection import NoConnectException
from com.sun.star.lang import DisposedException


def prop(name: str, value: object) -> PropertyValue:
    p = PropertyValue()
    p.Name = name
    p.Value = value
    return p


def accept_all(src: Path, dst: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="lo_profile_") as profile:
        pipe = f"jubarte_example_{time.monotonic_ns()}"
        office = subprocess.Popen(
            [
                "soffice",
                "--headless",
                "--norestore",
                "--nologo",
                f"-env:UserInstallation={Path(profile).as_uri()}",
                f"--accept=pipe,name={pipe};urp;",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            resolver = (
                uno.getComponentContext().ServiceManager.createInstanceWithContext(
                    "com.sun.star.bridge.UnoUrlResolver", uno.getComponentContext()
                )
            )
            ctx = None
            for _ in range(120):
                try:
                    ctx = resolver.resolve(
                        f"uno:pipe,name={pipe};urp;StarOffice.ComponentContext"
                    )
                    break
                except NoConnectException:
                    time.sleep(0.5)
            if ctx is None:
                raise SystemExit("LibreOffice did not start")
            smgr = ctx.ServiceManager
            desktop = smgr.createInstanceWithContext("com.sun.star.frame.Desktop", ctx)
            shutil.copyfile(src, dst)
            url = uno.systemPathToFileUrl(str(dst.resolve()))
            doc = desktop.loadComponentFromURL(
                url, "_blank", 0, (prop("Hidden", True),)
            )
            dispatcher = smgr.createInstanceWithContext(
                "com.sun.star.frame.DispatchHelper", ctx
            )
            frame = doc.getCurrentController().getFrame()
            dispatcher.executeDispatch(frame, ".uno:AcceptAllTrackedChanges", "", 0, ())
            doc.storeToURL(url, (prop("FilterName", "MS Word 2007 XML"),))
            doc.close(True)
            # The office closes the bridge while it terminates.
            with contextlib.suppress(DisposedException):
                desktop.terminate()
        finally:
            try:
                office.wait(timeout=30)
            except subprocess.TimeoutExpired:
                office.kill()


if __name__ == "__main__":
    accept_all(Path(sys.argv[1]), Path(sys.argv[2]))
