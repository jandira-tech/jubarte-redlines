# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte.mcp_server`` under its former name: the same module object."""

import sys

import jubarte.mcp_server

sys.modules[__name__] = jubarte.mcp_server
