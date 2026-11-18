# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte.document`` under its former name: the same module object."""

import sys

import jubarte.document

sys.modules[__name__] = jubarte.document
