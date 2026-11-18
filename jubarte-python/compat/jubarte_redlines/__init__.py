# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Former import name of ``jubarte`` (PyPI ``jubarte-docx``).

Every name is ``jubarte``'s own object: ``jubarte_redlines.compare_documents
is jubarte.compare_documents``. The submodules (``models``, ``document``,
``mcp_server``, ``_native``) are ``jubarte``'s modules under this name.
"""

import jubarte
from jubarte import *  # noqa: F403
from jubarte import __all__, __version__  # noqa: F401


def __getattr__(name: str) -> object:
    return getattr(jubarte, name)
