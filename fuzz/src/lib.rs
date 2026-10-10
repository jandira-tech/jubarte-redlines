// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shared allocator configuration for native fuzz harnesses.
//! C address-sanitizer instrumentation and mimalloc's ASan tracking must be
//! enabled together via `scripts/fuzz-native.sh`.

#[cfg(feature = "native-allocator")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
