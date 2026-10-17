# Functions and methods in `src/convert/preset_geom.rs`

15 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `Fill::shade`  (L35-45, pub(crate))
`pub(crate) fn shade(self, base: [f32; 3]) -> Option<[f32; 3]>`

- **Does:** Word's shade of `base` for this path (ECMA-376 ST_PathFillMode): darken 60%, darkenLess 80%, lighten 40% / lightenLess 20% toward white.
- **Calls (std/external/unresolved):** `.map`, `Some`, `mix`
- **Called by:** `Layout::paint_preset_paths` (convert/mod.rs:28202) (?); `tests::fill_modes_shade_toward_black_or_white` (convert/preset_geom.rs:696) (?)

#### `Cmd::view`  (L87-96)
`fn view(&self) -> CmdRef<'static>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `CmdRef::A`, `CmdRef::C`, `CmdRef::L`, `CmdRef::M`, `CmdRef::Q`
- **Called by:** none found in src

#### `OwnedCmd::view`  (L110-119)
`fn view(&self) -> CmdRef<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.each_ref`, `.map`, `CmdRef::A`, `CmdRef::C`, `CmdRef::L`, `CmdRef::M`, `CmdRef::Q`
- **Called by:** none found in src

#### `text_rect`  (L170-175, pub(crate))
`pub(crate) fn text_rect(name: &str, w: f32, h: f32, adj: &[(String, f64)]) -> Option<[f32; 4]>`

- **Does:** The text rectangle `[l, t, r, b]` (points, y down) of preset `name` in a `w`×`h` box, or `None` when it is the whole box.
- **Calls (crate):** `guides` (convert/preset_geom.rs:191) L172; `operand` (convert/preset_geom.rs:240) L173
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`, `Some`, `f64::from`
- **Called by:** `Layout::emit_textbox_paras` (convert/mod.rs:29178); `tests::regression_tests::text_rectangle_uses_shape_dimensions_in_points` (convert/preset_geom.rs:707); `tests::regression_tests::pie_and_gear_text_rectangles_match_word` (convert/preset_geom.rs:721); `tests::regression_tests::adjusted_text_rectangle_clamps_to_the_shapes_limits` (convert/preset_geom.rs:741)

#### `preset`  (L177-179, pub(crate))
`pub(crate) fn preset(name: &str) -> Option<&'static Preset>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `tests::eval` (convert/preset_geom.rs:505); `tests::avlst_overrides_the_default_adjustment` (convert/preset_geom.rs:685)

#### `ang_rad`  (L182-184)
`fn ang_rad(v: f64) -> f64`

- **Does:** DrawingML angles are 60000ths of a degree.
- **Calls (std/external/unresolved):** `.to_radians`
- **Called by:** `formula` (convert/preset_geom.rs:248); `evaluate_views` (convert/preset_geom.rs:357)

#### `rad_ang`  (L186-188)
`fn rad_ang(r: f64) -> f64`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_degrees`
- **Called by:** `formula` (convert/preset_geom.rs:248)

#### `guides`  (L191-238)
`fn guides( av: &[(&str, &str)], gd: &[(&str, &str)], w: f64, h: f64, adj: &[(String, f64)], ) -> HashMap<String, f64>`

- **Does:** Guide values for a `w`×`h` shape with `adj` overrides (`a:avLst`).
- **Calls (crate):** `formula` (convert/preset_geom.rs:248) L230,234
- **Calls (std/external/unresolved):** `.find`, `.insert`, `.iter`, `.map_or_else`, `.max`, `.min`, `.to_string`, `HashMap::new`, `f64::from`
- **Macros:** `format!`
- **Called by:** `cust_geom` (convert/mod.rs:16963); `text_rect` (convert/preset_geom.rs:170); `evaluate_views` (convert/preset_geom.rs:357)

#### `operand`  (L240-245)
`fn operand(tok: &str, g: &HashMap<String, f64>) -> f64`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.ok`, `.or_else`, `.parse`, `.unwrap_or`
- **Called by:** `text_rect` (convert/preset_geom.rs:170); `formula` (convert/preset_geom.rs:248); `evaluate_views` (convert/preset_geom.rs:357)

#### `formula`  (L248-300)
`fn formula(fmla: &str, g: &HashMap<String, f64>) -> f64`

- **Does:** ECMA-376 §20.1.9.11 guide formula.
- **Calls (crate):** `operand` (convert/preset_geom.rs:240) L251; `rad_ang` (convert/preset_geom.rs:186) L279; `ang_rad` (convert/preset_geom.rs:182) L282,283,284
- **Calls (std/external/unresolved):** `.abs`, `.atan2`, `.collect`, `.copied`, `.cos`, `.first`, `.get`, `.map`, `.max`, `.min`, `.next`, `.sin`, `.split_whitespace`, `.sqrt`, `.tan`, `.unwrap_or`
- **Called by:** `guides` (convert/preset_geom.rs:191); `tests::formulas_follow_ecma_20_1_9_11` (convert/preset_geom.rs:582); `residual_custom_path_contract_tests::guide_division_and_conditional_boundaries_keep_finite_authored_coordinates` (convert/preset_geom.rs:758)

#### `param_angle`  (L304-306)
`fn param_angle(a: f64, wr: f64, hr: f64) -> f64`

- **Does:** The parametric angle of the point at ray angle `a` on an ellipse with radii `wr`, `hr` (DrawingML arc angles are ray angles, not parameters).
- **Calls (std/external/unresolved):** `.atan2`, `.cos`, `.sin`
- **Called by:** `evaluate_views` (convert/preset_geom.rs:357)

#### `evaluate`  (L309-322, pub(crate))
`pub(crate) fn evaluate(preset: &Preset, w: f32, h: f32, adj: &[(String, f64)]) -> Vec<EvalPath>`

- **Does:** Evaluate `preset` for a `w`×`h` box (points) with `a:avLst` overrides.
- **Calls (crate):** `evaluate_views` (convert/preset_geom.rs:357) L321
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `tests::eval` (convert/preset_geom.rs:505); `tests::avlst_overrides_the_default_adjustment` (convert/preset_geom.rs:685)

#### `evaluate_custom`  (L329-353, pub(crate))
`pub(crate) fn evaluate_custom(geom: &CustomGeom, w: f32, h: f32) -> Vec<EvalPath>`

- **Does:** Evaluate a document's `a:custGeom` for a `w`×`h` box (points).
- **Calls (crate):** `evaluate_views` (convert/preset_geom.rs:357) L344; `evaluate_custom::{fn pairs}` (convert/preset_geom.rs:330) L345,346
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Layout::paint_box_geom` (convert/mod.rs:28243); `field_group_coordinate_contract_tests::intentionally_incomplete_custom_beziers_do_not_drop_later_valid_path_commands` (convert/mod.rs:63182); `tests::custom_guides_live_in_the_shapes_emu_space` (convert/preset_geom.rs:527); `tests::a_custom_arc_sweep_is_tessellated_within_a_bound` (convert/preset_geom.rs:555); `residual_custom_path_contract_tests::guide_division_and_conditional_boundaries_keep_finite_authored_coordinates` (convert/preset_geom.rs:758); `residual_custom_path_contract_tests::cubic_and_quadratic_paths_keep_literal_midpoints_endpoints_and_independent_flags` (convert/preset_geom.rs:811); `residual_custom_path_contract_tests::clockwise_and_counterclockwise_custom_arcs_keep_authored_radii_and_bounded_turns` (convert/preset_geom.rs:861)

#### `evaluate_custom::{fn pairs}`  (L330-332)
`fn pairs(v: &[(String, String)]) -> Vec<(&str, &str)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `evaluate_custom` (convert/preset_geom.rs:329)

#### `evaluate_views`  (L357-498)
`fn evaluate_views( av: &[(&str, &str)], gd: &[(&str, &str)], paths: &[PathView<'_>], w: f32, h: f32, adj: &[(String, f64)], units: f64, ) -> Vec<EvalPath>`

- **Does:** `units` is guide units per point: presets keep points (1), a custGeom works in EMU, and a path without its own w/h takes the guides' space.
- **Calls (crate):** `guides` (convert/preset_geom.rs:191) L367; `operand` (convert/preset_geom.rs:240) L368; `pt` (markdown/from_docx/mermaid.rs:278) (by name) L387,403,435,459,477; `ang_rad` (convert/preset_geom.rs:182) L408; `param_angle` (convert/preset_geom.rs:304) L409,411
- **Calls (std/external/unresolved):** `.abs`, `.ceil`, `.cos`, `.is_empty`, `.is_finite`, `.last_mut`, `.len`, `.max`, `.min`, `.push`, `.retain`, `.sin`, `.trunc`, `Vec::new`, `begin`, `f64::from`, `val`
- **Macros:** `vec!`
- **Called by:** `evaluate` (convert/preset_geom.rs:309); `evaluate_custom` (convert/preset_geom.rs:329)

## Test functions (19)

- `tests::eval` (L505): _no doc comment_
- `tests::bounds` (L509): _no doc comment_
- `tests::custom_guides_live_in_the_shapes_emu_space` (L527): _no doc comment_
- `tests::a_custom_arc_sweep_is_tessellated_within_a_bound` (L555): _no doc comment_
- `tests::formulas_follow_ecma_20_1_9_11` (L582): _no doc comment_
- `tests::up_down_arrow_keeps_a_shaft_on_wide_extents` (L595): _no doc comment_
- `tests::donut_is_two_closed_contours` (L617): _no doc comment_
- `tests::brackets_stroke_an_open_path` (L624): _no doc comment_
- `tests::cube_shades_its_faces_and_strokes_one_outline` (L646): _no doc comment_
- `tests::circular_arrow_stays_inside_its_extent` (L658): _no doc comment_
- `tests::moon_is_a_crescent_inside_its_box` (L674): _no doc comment_
- `tests::avlst_overrides_the_default_adjustment` (L685): _no doc comment_
- `tests::fill_modes_shade_toward_black_or_white` (L696): _no doc comment_
- `tests::regression_tests::text_rectangle_uses_shape_dimensions_in_points` (L707): _no doc comment_
- `tests::regression_tests::pie_and_gear_text_rectangles_match_word` (L721): _no doc comment_
- `tests::regression_tests::adjusted_text_rectangle_clamps_to_the_shapes_limits` (L741): _no doc comment_
- `residual_custom_path_contract_tests::guide_division_and_conditional_boundaries_keep_finite_authored_coordinates` (L758): _no doc comment_
- `residual_custom_path_contract_tests::cubic_and_quadratic_paths_keep_literal_midpoints_endpoints_and_independent_flags` (L811): _no doc comment_
- `residual_custom_path_contract_tests::clockwise_and_counterclockwise_custom_arcs_keep_authored_radii_and_bounded_turns` (L861): _no doc comment_
