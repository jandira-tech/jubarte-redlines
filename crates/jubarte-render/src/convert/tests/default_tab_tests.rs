// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn a_tab_on_a_default_stop_goes_to_the_next_one() {
    // 0081ba58: three tabs after a first one landed on 3 x 35.4 stayed
    // there (the f32 quotient was 2.9999), losing two stops.
    let origin = 63.8;
    let on_stop = origin + 3.0 * 35.4;
    let next = next_tab_x(on_stop, origin, &[], 35.4);
    assert!((next - (origin + 4.0 * 35.4)).abs() < 0.01, "{next}");
}
