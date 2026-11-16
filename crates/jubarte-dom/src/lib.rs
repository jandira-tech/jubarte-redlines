// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-dom
//!
//! The document-object-model floor of the jubarte redline engine: the
//! arena XML DOM (`xmllinq`), WordprocessingML namespace and `XName`
//! constants (`namespaces`), short ids and unique ids (`ids`, `unid`),
//! the compiled-out performance counters (`perf`), shared small utilities
//! (`util`), and the OOXML vocabulary notes and input limits the layers
//! above share (`ooxml`, `limits`).
//!
//! Nothing here knows Word documents as packages; the OPC adapter and
//! every product above it live in higher jubarte crates and reach this
//! one downward only.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ids;
/// Bound untrusted input before it is opened. Private in the facade; the
/// `jubarte-opc` crate consumes it (`jubarte::admission` re-exports the
/// limits types).
// TODO(split): `#[doc(hidden)]` once jubarte-opc exists to name it (A5 V2).
#[doc(hidden)]
pub mod limits;
/// WordprocessingML and related namespace / `XName` constants.
pub mod namespaces;
/// OOXML vocabulary facts shared by the layers above. Private in the
/// facade (`jubarte::opc` and friends hold their own).
#[doc(hidden)]
pub mod ooxml;
/// P0-LAB-01 stage counters/timers — no-ops unless `perf-profile` is
/// enabled.
pub mod perf;
pub mod unid;
/// Shared small utilities.
pub mod util;
/// Arena DOM (`xmllinq`) used by the comparer.
pub mod xmllinq;
