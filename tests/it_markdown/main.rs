// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

#[path = "../common/mod.rs"]
mod common;

mod agent_view_attribution;
mod cell_inside_container;
mod choice_requires;
mod comment_body_change;
mod docx_markdown_round_trip;
mod docx_to_markdown_critic_markup;
mod docx_to_markdown_critic_synthetic;
mod docx_to_markdown_office_samples;
mod fallback_inner_picture;
mod fmt_inside_insertion;
mod footnote_text_alias;
mod inspect_paragraphs;
mod markdown_diff;
mod markdown_page_size;
mod markdown_patch;
mod markdown_patch_docx;
mod markdown_to_docx;
mod md_agent_tag_patch;
mod md_author_separator;
mod md_comment_styles;
mod md_heading_emphasis;
mod md_heading_marks;
mod md_held_comment_marks;
mod md_mark_highlight;
mod md_math_stand_in;
mod md_math_word;
mod md_nested_changes;
mod md_note_separator_changes;
mod md_note_styles;
mod md_plain_legacy_attr;
mod md_re_tags;
mod md_reader_id_lines;
mod md_reader_threads;
mod md_review_1142;
mod md_revision_header;
mod md_roundtrip_valid;
mod md_run_size;
mod md_run_spaces;
mod md_section_grid;
mod md_story_fields;
mod md_story_format;
mod md_story_highlight;
mod md_story_styles;
mod md_story_trailing_paragraphs;
mod md_substitution_adjacency;
mod md_substitution_dates;
mod md_table_borders;
mod md_table_geometry;
mod md_tracked_page_break;
mod move_roundtrip;
mod nested_substitution;
mod symbol_roundtrip;
mod textbox_picture;
mod touching_changes;
mod tracked_break_place;
