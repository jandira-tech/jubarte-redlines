# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""``jubarte._native`` under its former name: the same module object."""

import sys

import jubarte._native

sys.modules[__name__] = jubarte._native
