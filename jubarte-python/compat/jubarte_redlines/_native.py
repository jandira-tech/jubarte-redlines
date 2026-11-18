# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte._native`` under its former name: the same module object."""

import sys

import jubarte._native

sys.modules[__name__] = jubarte._native
