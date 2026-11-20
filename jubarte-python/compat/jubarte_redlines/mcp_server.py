# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""``jubarte.mcp_server`` under its former name: the same module object."""

import sys

import jubarte.mcp_server

sys.modules[__name__] = jubarte.mcp_server
