use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, crossterm::event::KeyModifiers, style::Color};

use crate::{
    draw::{max_offset, render, reveal_offset},
    history::History,
    nav::{Loading, View, fit, follow, page, press},
    parse::parse,
};

const SEP: &str = "\x1f";
const NOW: i64 = 2_000_000;

fn line(hash: &str, parents: &str, refs: &str, date: &str, author: &str, subject: &str) -> String {
    [hash, parents, refs, date, author, subject].join(SEP)
}

fn draw(log: &str, width: u16, height: u16) -> Buffer {
    draw_after(log, width, height, "")
}

fn draw_after(log: &str, width: u16, height: u16, keys: &str) -> Buffer {
    let keys: Vec<(char, KeyModifiers)> = keys.chars().map(|key| (key, KeyModifiers::NONE)).collect();

    draw_chords(log, width, height, &keys)
}

fn draw_chords(log: &str, width: u16, height: u16, keys: &[(char, KeyModifiers)]) -> Buffer {
    draw_loading_chords(log, width, height, keys, &Loading::Done)
}

fn draw_loading(log: &str, width: u16, height: u16, keys: &str, loading: &Loading) -> Buffer {
    let keys: Vec<(char, KeyModifiers)> = keys.chars().map(|key| (key, KeyModifiers::NONE)).collect();

    draw_loading_chords(log, width, height, &keys, loading)
}

fn draw_loading_chords(log: &str, width: u16, height: u16, keys: &[(char, KeyModifiers)], loading: &Loading) -> Buffer {
    let mut history = History::default();
    for commit in parse(log) {
        history.push(commit);
    }
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let mut view = View::default();
    let mut frame_after = |view: View| {
        let view = fit(
            view,
            history.len(),
            usize::from(height),
            max_offset(history.max_lanes(), usize::from(width)),
        );
        terminal.draw(|frame| render(frame, &history, view, loading, NOW)).unwrap();

        view
    };
    view = frame_after(view);
    let page = page(usize::from(height), loading);
    for &(key, modifiers) in keys {
        let after = press(view, key, modifiers, history.len(), page);
        let node_column = history.lane_row(after.selected).node_column as usize;
        let node_offset = reveal_offset(after.offset, node_column, history.max_lanes(), usize::from(width));
        view = frame_after(follow(view, after, node_offset));
    }

    terminal.backend().buffer().clone()
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width).map(|x| buffer[(x, y)].symbol().to_string()).collect()
}

#[test]
fn shows_one_row_per_commit_newest_first() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "", "1913600", "Ann", "second"),
        line("aaaaaaaa", "", "", "1827200", "Ann", "first"),
    ]
    .join("\n");

    let buffer = draw(&log, 60, 3);

    assert!(row_text(&buffer, 0).contains("second"));
    assert!(row_text(&buffer, 1).contains("first"));
    assert_eq!(row_text(&buffer, 2).trim(), "");
}

fn two_commits() -> String {
    [
        line("1234567890abcdef", "fedcba0987654321", "", "1740800", "Ann", "short subject"),
        line("fedcba0987654321", "", "", "790400", "Bartholomew", "older"),
    ]
    .join("\n")
}

#[test]
fn aligns_graph_hash_date_author_and_message_columns() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(row_text(&buffer, 0).trim_end(), "● 1234567 3d   Ann             short subject");
    assert_eq!(row_text(&buffer, 1).trim_end(), "● fedcba0 14d  Bartholomew     older");
}

#[test]
fn truncates_an_author_wider_than_fifteen_cells_with_an_ellipsis() {
    let log = line("1234567890abcdef", "", "", "1913600", "Bartholomew Maximilian Fitzgerald", "subject");

    let buffer = draw(&log, 60, 1);

    assert_eq!(row_text(&buffer, 0).trim_end(), "● 1234567 1d   Bartholomew Ma… subject");
}

#[test]
fn shows_the_date_on_four_cells_whatever_the_loaded_commits() {
    let log = line("1234567890abcdef", "", "", "1999990", "Ann", "short subject");

    let buffer = draw(&log, 60, 1);

    assert_eq!(row_text(&buffer, 0).trim_end(), "● 1234567 now  Ann             short subject");
}

#[test]
fn draws_head_as_a_hollow_node_and_other_commits_as_filled_nodes() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "origin/main", "1913600", "Ann", "second"),
        line("aaaaaaaa", "", "HEAD -> main, tag: v1", "1827200", "Ann", "first"),
        line("cccccccc", "", "HEAD", "1740800", "Ann", "detached"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 0).starts_with('●'));
    assert!(row_text(&buffer, 1).starts_with('○'));
    assert!(row_text(&buffer, 2).starts_with('○'));
}

#[test]
fn aligns_columns_by_display_width_for_wide_characters() {
    let log = [
        line("1234567890abcdef", "", "", "1740800", "山田", "Wide"),
        line("fedcba0987654321", "", "", "1740800", "Ann", "Narrow"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    let column_of = |y: u16, symbol: &str| (0..40).find(|&x| buffer[(x, y)].symbol() == symbol);

    assert_eq!(column_of(0, "W"), column_of(1, "N"));
    assert!(column_of(0, "W").is_some());
}

#[test]
fn truncates_the_message_first_in_a_narrow_terminal() {
    let buffer = draw(&two_commits(), 40, 2);

    assert_eq!(row_text(&buffer, 0), "● 1234567 3d   Ann             short su…");
}

#[test]
fn colours_hash_and_date_in_overlay_and_text_in_text_and_subtext() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(buffer[(2, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
    assert_eq!(buffer[(10, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
    assert_eq!(buffer[(36, 0)].fg, Color::Rgb(0xcd, 0xd6, 0xf4));
    assert_eq!(buffer[(36, 1)].bg, Color::Reset);
}

#[test]
fn colours_author_in_subtext_apart_from_the_date() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(buffer[(22, 0)].fg, Color::Rgb(0xa6, 0xad, 0xc8));
    assert_eq!(buffer[(22, 1)].fg, Color::Rgb(0xa6, 0xad, 0xc8));
}

const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);
const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);

fn five_commits() -> String {
    ["e", "d", "c", "b", "a"]
        .map(|name| line(&name.repeat(8), "", "", "1913600", "Ann", &format!("commit {name}")))
        .join("\n")
}

fn highlighted_rows(buffer: &Buffer) -> Vec<u16> {
    (0..buffer.area.height).filter(|&y| buffer[(0, y)].bg == SURFACE0).collect()
}

#[test]
fn highlights_the_first_row_in_surface0_at_start() {
    let buffer = draw(&five_commits(), 40, 5);

    assert_eq!(highlighted_rows(&buffer), vec![0]);
    assert_eq!(buffer[(39, 0)].bg, SURFACE0);
    assert_eq!(buffer[(2, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
}

#[test]
fn bolds_the_subject_of_the_selected_row_only() {
    let buffer = draw(&five_commits(), 40, 5);

    assert!(is_bold(&buffer, 0, "commit e"));
    assert!(!is_bold(&buffer, 1, "commit d"));
    assert!(!is_bold(&buffer, 0, "Ann"));
}

#[test]
fn j_moves_the_highlight_down_one_commit() {
    let buffer = draw_after(&five_commits(), 40, 5, "jj");

    assert_eq!(highlighted_rows(&buffer), vec![2]);
}

#[test]
fn j_scrolls_the_view_to_keep_the_selection_visible() {
    let buffer = draw_after(&five_commits(), 60, 3, "jjjj");

    assert!(row_text(&buffer, 0).contains("commit c"));
    assert!(row_text(&buffer, 2).contains("commit a"));
    assert_eq!(highlighted_rows(&buffer), vec![2]);
}

#[test]
fn j_on_the_last_commit_does_nothing() {
    let at_last = draw_after(&five_commits(), 40, 3, "jjjj");
    let past_last = draw_after(&five_commits(), 40, 3, "jjjjjjj");

    assert_eq!(at_last, past_last);
}

#[test]
fn k_moves_the_highlight_up_one_commit() {
    let buffer = draw_after(&five_commits(), 40, 5, "jjjk");

    assert_eq!(highlighted_rows(&buffer), vec![2]);
}

#[test]
fn k_scrolls_the_view_back_up_to_keep_the_selection_visible() {
    let buffer = draw_after(&five_commits(), 60, 3, "jjjjkkk");

    assert!(row_text(&buffer, 0).contains("commit d"));
    assert_eq!(highlighted_rows(&buffer), vec![0]);
}

#[test]
fn k_on_the_first_commit_does_nothing() {
    let at_first = draw_after(&five_commits(), 40, 3, "");
    let before_first = draw_after(&five_commits(), 40, 3, "kk");

    assert_eq!(at_first, before_first);
}

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const NONE: KeyModifiers = KeyModifiers::NONE;

fn twelve_commits() -> String {
    ["l", "k", "j", "i", "h", "g", "f", "e", "d", "c", "b", "a"]
        .map(|name| line(&name.repeat(8), "", "", "1913600", "Ann", &format!("commit {name}")))
        .join("\n")
}

#[test]
fn capital_g_highlights_the_last_commit() {
    let buffer = draw_after(&twelve_commits(), 60, 4, "G");

    assert!(row_text(&buffer, 3).contains("commit a"));
    assert_eq!(highlighted_rows(&buffer), vec![3]);
}

#[test]
fn g_after_capital_g_highlights_the_first_commit() {
    let buffer = draw_after(&twelve_commits(), 60, 4, "Gg");

    assert!(row_text(&buffer, 0).contains("commit l"));
    assert_eq!(highlighted_rows(&buffer), vec![0]);
}

#[test]
fn ctrl_d_moves_the_highlight_down_half_a_page() {
    let buffer = draw_chords(&twelve_commits(), 60, 4, &[('d', CTRL)]);

    assert_eq!(highlighted_rows(&buffer), vec![2]);
    assert!(row_text(&buffer, 2).contains("commit j"));
}

#[test]
fn ctrl_u_moves_the_highlight_up_half_a_page() {
    let buffer = draw_chords(&twelve_commits(), 60, 4, &[('d', CTRL), ('d', CTRL), ('u', CTRL)]);

    assert!(row_text(&buffer, highlighted_rows(&buffer)[0]).contains("commit j"));
}

#[test]
fn ctrl_d_and_ctrl_u_stop_at_the_edges_of_the_list() {
    let at_last = draw_after(&twelve_commits(), 60, 4, "G");
    let past_last = draw_chords(&twelve_commits(), 60, 4, &[('G', NONE), ('d', CTRL), ('d', CTRL)]);
    let at_first = draw_after(&twelve_commits(), 60, 4, "");
    let before_first = draw_chords(&twelve_commits(), 60, 4, &[('u', CTRL)]);

    assert_eq!(at_last, past_last);
    assert_eq!(at_first, before_first);
}

#[test]
fn g_on_the_first_commit_and_capital_g_on_the_last_do_nothing() {
    assert_eq!(draw_after(&twelve_commits(), 60, 4, ""), draw_after(&twelve_commits(), 60, 4, "g"));
    assert_eq!(draw_after(&twelve_commits(), 60, 4, "G"), draw_after(&twelve_commits(), 60, 4, "GG"));
}

#[test]
fn d_and_u_without_ctrl_do_nothing() {
    let start = draw_after(&twelve_commits(), 60, 4, "");

    assert_eq!(start, draw_after(&twelve_commits(), 60, 4, "du"));
    assert_eq!(draw_after(&twelve_commits(), 60, 4, "jjj"), draw_after(&twelve_commits(), 60, 4, "jjjdu"));
}

#[test]
fn a_half_page_is_at_least_one_row() {
    let buffer = draw_chords(&twelve_commits(), 60, 1, &[('d', CTRL)]);

    assert!(row_text(&buffer, 0).contains("commit k"));
}

const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);
const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
const PINK: Color = Color::Rgb(0xf5, 0xc2, 0xe7);
const TEAL: Color = Color::Rgb(0x94, 0xe2, 0xd5);
const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);

fn graph(buffer: &Buffer, y: u16, width: u16) -> String {
    (0..width).map(|x| buffer[(x, y)].symbol().to_string()).collect()
}

fn branch_log() -> String {
    [
        line("ccccccc", "bbbbbbb", "", "1913600", "Ann", "c"),
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "b"),
        line("sssssss", "aaaaaaa", "", "1913600", "Ann", "s"),
        line("aaaaaaa", "", "", "1913600", "Ann", "a"),
    ]
    .join("\n")
}

#[test]
fn keeps_the_first_parent_chain_in_the_same_lane() {
    let buffer = draw(&branch_log(), 40, 4);

    assert_eq!(graph(&buffer, 0, 3), "●  ");
    assert_eq!(graph(&buffer, 1, 3), "●  ");
    assert_eq!(graph(&buffer, 3, 1), "●");
}

#[test]
fn opens_a_lane_for_a_branch_and_joins_the_lane_of_its_parent() {
    let buffer = draw(&branch_log(), 40, 4);

    assert_eq!(graph(&buffer, 2, 3), "│ ●");
    assert_eq!(graph(&buffer, 3, 3), "●─╯");
}

fn merge_log() -> String {
    [
        line("mmmmmmm", "bbbbbbb sssssss", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "b"),
        line("sssssss", "aaaaaaa", "", "1913600", "Ann", "s"),
        line("aaaaaaa", "", "", "1913600", "Ann", "a"),
    ]
    .join("\n")
}

#[test]
fn opens_a_lane_for_the_second_parent_of_a_merge() {
    let buffer = draw(&merge_log(), 40, 4);

    assert_eq!(graph(&buffer, 0, 3), "●─╮");
    assert_eq!(graph(&buffer, 1, 3), "● │");
    assert_eq!(graph(&buffer, 2, 3), "│ ●");
    assert_eq!(graph(&buffer, 3, 3), "●─╯");
}

#[test]
fn opens_a_lane_for_each_parent_after_the_first_of_an_octopus_merge() {
    let log = [
        line("mmmmmmm", "bbbbbbb sssssss ttttttt", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "", "", "1913600", "Ann", "b"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    assert_eq!(graph(&buffer, 0, 5), "●─╮─╮");
    assert_eq!(graph(&buffer, 1, 5), "● │ │");
}

fn tips(count: usize) -> String {
    (0..count)
        .map(|n| line(&format!("tip{n:04}"), "", "", "1913600", "Ann", "t"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn open_lanes_log(count: usize) -> String {
    let parents: Vec<String> = (0..count).map(|n| format!("par{n:04}")).collect();

    line("mmmmmmm", &parents.join(" "), "", "1913600", "Ann", "m")
}

#[test]
fn colours_each_lane_in_the_mocha_cycle() {
    let buffer = draw(&open_lanes_log(8), 120, 1);
    let colours: Vec<Color> = (0..8).map(|column| buffer[(2 * column, 0)].fg).collect();

    assert_eq!(colours, [MAUVE, BLUE, GREEN, PEACH, PINK, TEAL, YELLOW, RED]);
}

#[test]
fn restarts_the_cycle_after_the_ninth_lane() {
    let buffer = draw(&open_lanes_log(9), 120, 1);

    assert_eq!(buffer[(16, 0)].fg, MAUVE);
}

#[test]
fn a_lane_keeps_one_colour_along_its_whole_length() {
    let buffer = draw(&merge_log(), 40, 4);

    assert_eq!(buffer[(2, 0)].fg, BLUE);
    assert_eq!(buffer[(2, 1)].fg, BLUE);
    assert_eq!(buffer[(2, 2)].fg, BLUE);
    assert_eq!(buffer[(0, 1)].fg, MAUVE);
}

#[test]
fn a_lane_reusing_a_freed_column_takes_the_next_colour_of_the_cycle() {
    let log = format!("{}\n{}", merge_log(), tips(1));

    let buffer = draw(&log, 40, 5);

    assert_eq!(buffer[(0, 4)].fg, GREEN);
}

#[test]
fn links_commits_to_parents_given_as_full_hashes() {
    let full = |short: &str| format!("{short}{}", "0".repeat(33));
    let log = [
        line(&full("1111111"), &full("2222222"), "", "1913600", "Ann", "tip"),
        line(&full("2222222"), "", "", "1913600", "Ann", "root"),
        line(&full("3333333"), "", "", "1913600", "Ann", "other"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 1).starts_with("● 2222222"));
}

#[test]
fn does_not_link_commits_sharing_their_first_seven_characters() {
    let full = |tail: &str| format!("bbbbbbb{tail}{}", "0".repeat(32));
    let log = [
        line(&format!("aaaaaaa1{}", "0".repeat(32)), &full("1"), "", "1913600", "Ann", "child"),
        line(&full("2"), "", "", "1913600", "Ann", "unrelated"),
        line(&full("1"), "", "", "1913600", "Ann", "parent"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert_eq!(graph(&buffer, 1, 3), "│ ●");
    assert_eq!(graph(&buffer, 2, 3), "●  ");
}

#[test]
fn a_new_lane_takes_the_leftmost_free_column() {
    let log = [
        line("xxxxxxx", "x000000", "", "1913600", "Ann", "x"),
        line("yyyyyyy", "", "", "1913600", "Ann", "y"),
        line("wwwwwww", "w000000", "", "1913600", "Ann", "w"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert_eq!(graph(&buffer, 1, 3), "│ ●");
    assert_eq!(graph(&buffer, 2, 3), "│ ●");
}

fn badge_log(refs: &str) -> String {
    line("aaaaaaa", "", refs, "1913600", "Ann", "subject")
}

const BASE: Color = Color::Rgb(0x1e, 0x1e, 0x2e);
const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
const SUBTEXT: Color = Color::Rgb(0xa6, 0xad, 0xc8);

fn column_of(buffer: &Buffer, y: u16, text: &str) -> u16 {
    let row = row_text(buffer, y);
    let x = row.find(text).unwrap_or_else(|| panic!("{text} not in {row}"));

    row[..x].chars().count() as u16
}

fn cell_at<'a>(buffer: &'a Buffer, y: u16, text: &str) -> &'a ratatui::buffer::Cell {
    &buffer[(column_of(buffer, y, text), y)]
}

#[test]
fn shows_the_refs_of_a_commit_as_pills_before_its_message() {
    let log = badge_log("HEAD -> refs/heads/main, refs/remotes/origin/main, refs/remotes/origin/HEAD, tag: refs/tags/v1.0");

    let buffer = draw(&log, 80, 1);

    assert_eq!(
        row_text(&buffer, 0).trim_end(),
        "○ aaaaaaa 1d   Ann              main   origin/main   ◆ v1.0  subject"
    );
}

#[test]
fn paints_a_pill_on_its_name_between_two_spaces_and_leaves_the_separator_unpainted() {
    let log = [
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "child"),
        badge_log("refs/heads/main, refs/remotes/origin/main"),
    ]
    .join("\n");
    let buffer = draw(&log, 80, 2);

    let start = column_of(&buffer, 1, " main ");
    let backgrounds: Vec<Color> = (start..start + 6).map(|x| buffer[(x, 1)].bg).collect();
    assert!(backgrounds.iter().all(|bg| *bg == backgrounds[0] && *bg != Color::Reset));
    assert_eq!(buffer[(start + 6, 1)].bg, Color::Reset);
    assert_eq!(buffer[(start + 7, 1)].bg, SURFACE1);
    let end = column_of(&buffer, 1, "origin/main ") + 11;
    assert_eq!(buffer[(end, 1)].bg, SURFACE1);
    assert_eq!(buffer[(end + 1, 1)].bg, Color::Reset);
}

#[test]
fn keeps_the_pill_background_on_the_selected_row() {
    let log = badge_log("refs/heads/main, refs/remotes/origin/main, tag: refs/tags/v1.0");
    let buffer = draw(&log, 80, 1);

    assert_eq!(cell_at(&buffer, 0, "main").bg, MAUVE);
    assert_eq!(cell_at(&buffer, 0, "origin/main").bg, SURFACE1);
    assert_eq!(cell_at(&buffer, 0, "v1.0").bg, YELLOW);
    assert_eq!(buffer[(79, 0)].bg, SURFACE0);
}

#[test]
fn colours_a_local_branch_pill_in_the_lane_colour_of_its_commit_with_dark_text() {
    let log = [
        line("ccccccc", "bbbbbbb", "", "1913600", "Ann", "c"),
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "b"),
        line("sssssss", "aaaaaaa", "refs/heads/side", "1913600", "Ann", "s"),
        line("aaaaaaa", "", "", "1913600", "Ann", "a"),
    ]
    .join("\n");

    let buffer = draw(&log, 60, 4);

    let pill = cell_at(&buffer, 2, "side");
    assert_eq!(pill.bg, BLUE);
    assert_eq!(pill.fg, BASE);
}

#[test]
fn colours_a_remote_branch_pill_in_surface1_with_subtext() {
    let log = [
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "child"),
        badge_log("refs/remotes/origin/main"),
    ]
    .join("\n");
    let buffer = draw(&log, 80, 2);

    let pill = cell_at(&buffer, 1, "origin/main");
    assert_eq!(pill.bg, SURFACE1);
    assert_eq!(pill.fg, SUBTEXT);
}

#[test]
fn colours_a_tag_pill_in_yellow_with_dark_text_and_keeps_its_diamond() {
    let buffer = draw(&badge_log("tag: refs/tags/v1.0"), 80, 1);

    let pill = cell_at(&buffer, 0, "◆ v1.0");
    assert_eq!(pill.bg, YELLOW);
    assert_eq!(pill.fg, BASE);
    assert_eq!(cell_at(&buffer, 0, "v1.0").bg, YELLOW);
}

fn is_bold(buffer: &Buffer, y: u16, text: &str) -> bool {
    cell_at(buffer, y, text).modifier.contains(ratatui::style::Modifier::BOLD)
}

#[test]
fn shows_the_branch_of_head_in_bold_without_a_head_pill() {
    let buffer = draw(&badge_log("HEAD -> refs/heads/main"), 80, 1);

    let row = row_text(&buffer, 0);
    assert!(!row.contains("HEAD"));
    assert!(row.starts_with("○ "));
    assert!(is_bold(&buffer, 0, "main"));
    assert_eq!(cell_at(&buffer, 0, "main").bg, MAUVE);
    assert_eq!(cell_at(&buffer, 0, "main").fg, BASE);
}

#[test]
fn bolds_only_the_branch_of_head_among_the_local_branches_of_its_commit() {
    let buffer = draw(&badge_log("refs/heads/alpha, HEAD -> refs/heads/zeta, refs/heads/beta"), 80, 1);

    assert_eq!(
        row_text(&buffer, 0).trim_end(),
        "○ aaaaaaa 1d   Ann              zeta   alpha   beta  subject"
    );
    assert!(is_bold(&buffer, 0, "zeta"));
    assert!(!is_bold(&buffer, 0, "alpha"));
    assert!(!is_bold(&buffer, 0, "beta"));
}

#[test]
fn shows_a_detached_head_as_a_bold_light_pill_with_dark_text() {
    let buffer = draw(&badge_log("refs/heads/zeta, HEAD"), 80, 1);

    assert!(column_of(&buffer, 0, "HEAD") < column_of(&buffer, 0, "zeta"));
    let pill = cell_at(&buffer, 0, "HEAD");
    assert!(row_text(&buffer, 0).starts_with("○ "));
    assert!(is_bold(&buffer, 0, "HEAD"));
    assert_eq!(pill.bg, TEXT);
    assert_eq!(pill.fg, BASE);
}

#[test]
fn gives_origin_head_no_badge() {
    let buffer = draw(&badge_log("refs/remotes/origin/HEAD"), 80, 1);

    assert!(!row_text(&buffer, 0).contains('['));
}

#[test]
fn leaves_the_terminal_background_unpainted_around_pills() {
    let log = [
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "child"),
        badge_log("refs/heads/main, tag: refs/tags/v1.0"),
    ]
    .join("\n");
    let buffer = draw(&log, 80, 2);

    let first_pill = column_of(&buffer, 1, " main ");
    let subject = column_of(&buffer, 1, "subject");
    assert!((0..first_pill).all(|x| buffer[(x, 1)].bg == Color::Reset));
    assert!((subject..80).all(|x| buffer[(x, 1)].bg == Color::Reset));
}

#[test]
fn truncates_the_message_after_the_pills_in_a_narrow_terminal() {
    let buffer = draw(&badge_log("refs/heads/main"), 43, 1);

    assert_eq!(row_text(&buffer, 0), "● aaaaaaa 1d   Ann              main  subj…");
}

#[test]
fn a_lane_passing_under_a_junction_keeps_its_vertical_line() {
    let log = [
        line("xxxxxxx", "aaaaaaa", "", "1913600", "Ann", "x"),
        line("yyyyyyy", "qqqqqqq", "", "1913600", "Ann", "y"),
        line("sssssss", "aaaaaaa", "", "1913600", "Ann", "s"),
        line("aaaaaaa", "", "", "1913600", "Ann", "a"),
        line("qqqqqqq", "", "", "1913600", "Ann", "q"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 5);

    assert_eq!(graph(&buffer, 3, 5), "●─│─╯");
    assert_eq!(buffer[(2, 3)].fg, BLUE);
}

#[test]
fn a_merge_parent_takes_the_column_freed_by_a_lane_ending_on_the_same_row() {
    let log = [
        line("xxxxxxx", "mmmmmmm", "", "1913600", "Ann", "x"),
        line("sssssss", "mmmmmmm", "", "1913600", "Ann", "s"),
        line("mmmmmmm", "bbbbbbb ppppppp", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "", "", "1913600", "Ann", "b"),
        line("ppppppp", "", "", "1913600", "Ann", "p"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 5);

    assert!(row_text(&buffer, 2).starts_with("●─┤ mmmmmmm"));
    assert_eq!(buffer[(2, 2)].fg, GREEN);
    assert_eq!(graph(&buffer, 3, 3), "● │");
}

#[test]
fn orders_pills_head_then_local_then_remote_then_tags_keeping_git_order_within_a_kind() {
    let log = badge_log("tag: refs/tags/v1, refs/remotes/origin/feature/a, refs/heads/zeta, refs/heads/feature/a, HEAD -> refs/heads/main");

    let buffer = draw(&log, 100, 1);

    assert_eq!(
        row_text(&buffer, 0).trim_end(),
        "○ aaaaaaa 1d   Ann              main   zeta   feature/a   origin/feature/a   ◆ v1  subject"
    );
}

#[test]
fn colours_each_junction_segment_like_the_corner_that_ends_it() {
    let log = [
        line("mmmmmmm", "bbbbbbb sssssss ttttttt", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "", "", "1913600", "Ann", "b"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    assert_eq!(graph(&buffer, 0, 5), "●─╮─╮");
    let colours: Vec<Color> = (1..5).map(|x| buffer[(x, 0)].fg).collect();
    assert_eq!(colours, [BLUE, BLUE, GREEN, GREEN]);
}

#[test]
fn draws_a_junction_across_four_lanes_without_a_gap() {
    let log = [
        line("mmmmmmm", "bbbbbbb sssssss ttttttt uuuuuuu", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "", "", "1913600", "Ann", "b"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    assert_eq!(graph(&buffer, 0, 7), "●─╮─╮─╮");
}

#[test]
fn opens_a_lane_left_of_the_node_with_an_opening_corner_facing_right() {
    let log = [
        line("xxxxxxx", "ppppppp", "", "1913600", "Ann", "x"),
        line("yyyyyyy", "mmmmmmm", "", "1913600", "Ann", "y"),
        line("ppppppp", "", "", "1913600", "Ann", "p"),
        line("mmmmmmm", "bbbbbbb sssssss", "", "1913600", "Ann", "m"),
        line("bbbbbbb", "", "", "1913600", "Ann", "b"),
        line("sssssss", "", "", "1913600", "Ann", "s"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 6);

    assert_eq!(graph(&buffer, 3, 3), "╭─●");
}

fn many_branches_log(count: usize) -> String {
    let tips = (0..count).map(|n| line(&format!("{n:07x}"), "ba5e000", "", "1913600", "Ann", &format!("branch {n}")));

    tips.chain([line("ba5e000", "", "", "1913600", "Ann", "base")])
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn limits_the_graph_column_to_a_quarter_of_the_terminal_so_the_text_stays_visible() {
    let buffer = draw(&many_branches_log(60), 80, 61);

    assert_eq!(row_text(&buffer, 59).chars().skip(21).take(7).collect::<String>(), "000003b");
    assert!(row_text(&buffer, 59).contains("branch 59"));
    assert!(row_text(&buffer, 60).contains("base"));
}

#[test]
fn shows_a_graph_that_fits_in_a_quarter_of_the_terminal_in_full() {
    let buffer = draw(&open_lanes_log(7), 52, 1);

    assert_eq!(graph(&buffer, 0, 13), "●─╮─╮─╮─╮─╮─╮");
}

fn scrolled_graph(keys: &str, y: u16) -> String {
    graph(&draw_after(&many_branches_log(6), 24, 7, keys), y, 6)
}

#[test]
fn l_shifts_the_graph_window_one_lane_to_the_right() {
    assert_eq!(scrolled_graph("", 4), "│ │ │ ");
    assert_eq!(scrolled_graph("l", 4), "│ │ │ ");
    assert_eq!(scrolled_graph("ll", 4), "│ │ ● ");
    assert_eq!(scrolled_graph("lll", 4), "│ ●   ");
}

#[test]
fn h_shifts_the_graph_window_one_lane_to_the_left() {
    assert_eq!(scrolled_graph("llh", 5), scrolled_graph("l", 5));
    assert_eq!(scrolled_graph("lllh", 5), scrolled_graph("ll", 5));
}

#[test]
fn h_at_the_left_edge_and_l_at_the_right_edge_do_nothing() {
    assert_eq!(scrolled_graph("hhh", 5), scrolled_graph("", 5));
    assert_eq!(scrolled_graph("llllll", 5), scrolled_graph("lll", 5));
    assert_eq!(scrolled_graph("llllllhh", 5), scrolled_graph("l", 5));
}

#[test]
fn the_graph_window_is_shifted_by_the_same_amount_on_every_row() {
    assert_eq!(scrolled_graph("lll", 3), "●     ");
    assert_eq!(scrolled_graph("lll", 4), "│ ●   ");
    assert_eq!(scrolled_graph("lll", 5), "│ │ ● ");
    assert_eq!(scrolled_graph("lll", 6), "╯─╯─╯ ");
}

fn dimmed_cells(buffer: &Buffer, y: u16, width: u16) -> Vec<u16> {
    (0..width).filter(|&x| buffer[(x, y)].fg == SURFACE1).collect()
}

#[test]
fn dims_the_two_edge_cells_on_the_side_of_hidden_lanes() {
    let log = many_branches_log(6);

    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, ""), 4, 6), [4, 5]);
    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "l"), 4, 6), [0, 1, 4, 5]);
    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "lll"), 4, 6), [0, 1]);
}

#[test]
fn keeps_the_glyphs_of_the_dimmed_cells_and_shows_no_ellipsis_in_the_graph() {
    let log = many_branches_log(6);

    assert_eq!(graph(&draw_after(&log, 24, 7, ""), 4, 6), "│ │ │ ");
    assert_eq!(graph(&draw_after(&log, 24, 7, "l"), 4, 6), "│ │ │ ");
    assert_eq!(graph(&draw_after(&log, 24, 7, "lll"), 4, 6), "│ ●   ");
}

#[test]
fn dims_the_edge_cells_on_every_row_whatever_their_lane_colour() {
    let buffer = draw_after(&many_branches_log(6), 24, 7, "l");

    for y in 0..7 {
        assert_eq!(dimmed_cells(&buffer, y, 6), [0, 1, 4, 5]);
    }
}

#[test]
fn dims_no_edge_when_the_graph_window_is_under_five_cells() {
    let buffer = draw_after(&many_branches_log(6), 16, 7, "l");

    assert_eq!(dimmed_cells(&buffer, 4, 4), Vec::<u16>::new());
}

#[test]
fn dims_no_edge_when_the_graph_fits_in_the_window() {
    let buffer = draw(&open_lanes_log(3), 40, 1);

    assert_eq!(dimmed_cells(&buffer, 0, 5), Vec::<u16>::new());
}

#[test]
fn h_and_l_keep_the_selection_and_j_and_k_bring_the_node_back() {
    let log = many_branches_log(6);

    assert_eq!(highlighted_rows(&draw_after(&log, 24, 7, "jjlh")), vec![2]);
    assert_eq!(draw_after(&log, 24, 7, "lljj"), draw_after(&log, 24, 7, "jj"));
    assert_eq!(draw_after(&log, 24, 7, "lljk"), draw_after(&log, 24, 7, ""));
}

fn parent_hash(n: usize, count: usize, last_parent: &str) -> String {
    if n + 1 == count {
        return last_parent.to_string();
    }

    format!("{:07x}", n + 1)
}

fn long_lane_log(mains: usize) -> String {
    let sides = [
        line("5100000", "r00t001", "", "1913600", "Ann", "side one"),
        line("5200000", "r00t002", "", "1913600", "Ann", "side two"),
    ];
    let chain = (0..mains).map(|n| {
        line(
            &format!("{n:07x}"),
            &parent_hash(n, mains, "r00t001"),
            "",
            "1913600",
            "Ann",
            &format!("main {n}"),
        )
    });

    sides
        .into_iter()
        .chain(chain)
        .chain([
            line("r00t001", "r00t002", "", "1913600", "Ann", "root one"),
            line("r00t002", "", "", "1913600", "Ann", "root two"),
        ])
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn keeps_two_lanes_open_from_the_first_row_to_the_last_across_the_resume_points() {
    let log = long_lane_log(1_100);

    for selected in [1_023, 1_024, 1_025] {
        let buffer = draw_after(&log, 60, 1, &"j".repeat(selected));

        assert_eq!(graph(&buffer, 0, 5), "│ │ ●", "selected {selected}");
        assert_eq!((buffer[(0, 0)].fg, buffer[(2, 0)].fg, buffer[(4, 0)].fg), (MAUVE, BLUE, GREEN));
    }
}

#[test]
fn draws_a_junction_across_a_free_column() {
    let log = [
        line("aaaaaaa", "ppppppp", "", "1913600", "Ann", "a"),
        line("bbbbbbb", "qqqqqqq", "", "1913600", "Ann", "b"),
        line("ccccccc", "ppppppp", "", "1913600", "Ann", "c"),
        line("qqqqqqq", "", "", "1913600", "Ann", "q"),
        line("ppppppp", "", "", "1913600", "Ann", "p"),
    ]
    .join("\n");

    let buffer = draw(&log, 60, 5);

    assert_eq!(graph(&buffer, 4, 5), "●───╯");
}

#[test]
fn leaves_a_free_column_left_of_a_junction_blank() {
    let log = [
        line("aaaaaaa", "xxxxxxx", "", "1913600", "Ann", "a"),
        line("bbbbbbb", "yyyyyyy", "", "1913600", "Ann", "b"),
        line("ccccccc", "yyyyyyy", "", "1913600", "Ann", "c"),
        line("xxxxxxx", "", "", "1913600", "Ann", "x"),
        line("yyyyyyy", "", "", "1913600", "Ann", "y"),
    ]
    .join("\n");

    let buffer = draw(&log, 60, 5);

    assert_eq!(graph(&buffer, 4, 5), "  ●─╯");
}

#[test]
fn dims_the_last_two_cells_when_two_cells_are_hidden() {
    let buffer = draw(&open_lanes_log(7), 44, 1);

    assert_eq!(graph(&buffer, 0, 11), "●─╮─╮─╮─╮─╮");
    assert_eq!(dimmed_cells(&buffer, 0, 11), [9, 10]);
}

#[test]
fn leaves_the_cells_past_the_graph_unpainted_at_the_right_edge() {
    let buffer = draw_after(&many_branches_log(6), 24, 7, "lll");

    assert_eq!(buffer[(5, 4)].fg, Color::Reset);
}

#[test]
fn draws_a_history_of_three_thousand_commits_from_its_last_row() {
    let log = (0..3_000)
        .map(|n| {
            line(
                &format!("{n:07x}"),
                &parent_hash(n, 3_000, ""),
                "",
                "1913600",
                "Ann",
                &format!("commit {n}"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let buffer = draw_after(&log, 60, 50, &"j".repeat(2_999));

    assert_eq!(graph(&buffer, 49, 1), "●");
    assert!(row_text(&buffer, 49).contains("commit 2999"));
    assert!(row_text(&buffer, 0).contains("commit 2950"));
}

fn three_commits() -> String {
    [
        line("ccccccc", "bbbbbbb", "", "1913600", "Ann", "third"),
        line("bbbbbbb", "aaaaaaa", "", "1913600", "Ann", "second"),
        line("aaaaaaa", "", "", "1913600", "Ann", "first"),
    ]
    .join("\n")
}

#[test]
fn shows_the_loading_count_in_overlay_on_the_last_row_while_reading() {
    let buffer = draw_loading(&three_commits(), 60, 4, "", &Loading::Reading);

    assert_eq!(row_text(&buffer, 3).trim_end(), "loading… 3 commits");
    assert_eq!(buffer[(0, 3)].fg, Color::Rgb(0x6c, 0x70, 0x86));
}

#[test]
fn gives_the_last_row_to_the_status_line_while_reading() {
    let buffer = draw_loading(&three_commits(), 60, 3, "", &Loading::Reading);

    assert!(row_text(&buffer, 1).contains("second"));
    assert!(row_text(&buffer, 2).starts_with("loading…"));
}

#[test]
fn shows_a_commit_on_the_last_row_once_done() {
    let buffer = draw_loading(&three_commits(), 60, 3, "", &Loading::Done);

    assert!(row_text(&buffer, 2).contains("first"));
}

#[test]
fn shows_the_first_line_of_the_failure_in_red_and_keeps_the_commits() {
    let failed = Loading::Failed("fatal: bad object\nsecond line".to_string());

    let buffer = draw_loading(&three_commits(), 60, 4, "", &failed);

    assert_eq!(row_text(&buffer, 3).trim_end(), "fatal: bad object");
    assert_eq!(buffer[(0, 3)].fg, Color::Rgb(0xf3, 0x8b, 0xa8));
    assert!(row_text(&buffer, 0).contains("third"));
}

fn selected_row(buffer: &Buffer) -> Option<u16> {
    (0..buffer.area.height).find(|&y| buffer[(0, y)].bg == SURFACE0)
}

#[test]
fn keeps_the_selection_on_the_same_commit_when_more_commits_arrive() {
    let log = three_commits();
    let mut history = History::default();
    let mut commits = parse(&log).into_iter();
    history.push(commits.next().unwrap());
    history.push(commits.next().unwrap());
    let mut terminal = Terminal::new(TestBackend::new(60, 5)).unwrap();
    let view = fit(press(View::default(), 'j', KeyModifiers::NONE, history.len(), 2), history.len(), 5, 0);
    terminal.draw(|frame| render(frame, &history, view, &Loading::Reading, NOW)).unwrap();
    let before = selected_row(terminal.backend().buffer());

    history.push(commits.next().unwrap());
    let view = fit(view, history.len(), 5, 0);
    terminal.draw(|frame| render(frame, &history, view, &Loading::Reading, NOW)).unwrap();
    let buffer = terminal.backend().buffer();

    assert_eq!(before, Some(1));
    assert_eq!(selected_row(buffer), before);
    assert!(row_text(buffer, 1).contains("second"));
}

fn node_cell(buffer: &Buffer, y: u16) -> Option<u16> {
    (0..6).find(|&x| matches!(buffer[(x, y)].symbol(), "●" | "○"))
}

fn assert_node_shown(buffer: &Buffer) {
    let y = selected_row(buffer).unwrap();
    let node = node_cell(buffer, y).unwrap_or_else(|| panic!("no node on the selected row {y}"));

    assert!(!dimmed_cells(buffer, y, 6).contains(&node), "node at {node} is in a dimmed edge");
}

#[test]
fn shows_the_node_of_the_selected_commit_after_each_j() {
    let log = many_branches_log(6);

    for presses in 0..=5 {
        assert_node_shown(&draw_after(&log, 24, 7, &"j".repeat(presses)));
    }
}

#[test]
fn shows_the_node_of_the_selected_commit_after_g_and_big_jumps() {
    let log = many_branches_log(6);
    let down = [('d', KeyModifiers::CONTROL)];
    let up = [('u', KeyModifiers::CONTROL)];

    assert_node_shown(&draw_chords(&log, 24, 7, &down));
    assert_node_shown(&draw_chords(&log, 24, 7, &[down[0], down[0]]));
    assert_node_shown(&draw_chords(&log, 24, 7, &[down[0], down[0], up[0]]));
    assert_node_shown(&draw_after(&log, 24, 7, "G"));
    assert_node_shown(&draw_after(&log, 24, 7, "Gg"));
    assert_node_shown(&draw_after(&log, 24, 7, "jjjjjg"));
}

#[test]
fn leaves_the_window_where_it_is_when_the_node_is_already_visible() {
    let log = many_branches_log(6);

    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "j"), 1, 6), [4, 5]);
    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "jjjjjk"), 4, 6), [0, 1]);
}

#[test]
fn shifts_the_window_by_the_least_amount_towards_the_node_on_either_side() {
    let log = many_branches_log(6);

    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "jj"), 2, 6), [0, 1, 4, 5]);
    assert_eq!(graph(&draw_after(&log, 24, 7, "jj"), 2, 6), "│ ●   ");
    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "jjjjjkk"), 3, 6), [0, 1, 4, 5]);
    assert_eq!(graph(&draw_after(&log, 24, 7, "jjjjjkk"), 3, 6), "│ ●   ");
}

#[test]
fn keeps_the_edge_on_the_side_of_hidden_lanes_after_a_follow() {
    let log = many_branches_log(6);

    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "jjjjj"), 5, 6), [0, 1]);
    assert_eq!(dimmed_cells(&draw_after(&log, 24, 7, "jjjjjg"), 0, 6), [4, 5]);
}

#[test]
fn changes_nothing_on_a_graph_that_fits_in_the_window() {
    let log = many_branches_log(3);

    assert_eq!(dimmed_cells(&draw_after(&log, 40, 4, "jjj"), 2, 5), Vec::<u16>::new());
    assert_eq!(graph(&draw_after(&log, 40, 4, "jj"), 2, 5), "│ │ ●");
}

fn is_revealed(offset: usize, cell: usize, graph_width: usize, window: usize) -> bool {
    let start = offset * 2;
    let left_edge = if window >= 5 && offset > 0 { 2 } else { 0 };
    let right_edge = if window >= 5 && start + window < graph_width { 2 } else { 0 };

    cell >= start + left_edge && cell + right_edge < start + window
}

#[test]
fn reveal_offset_is_the_closest_offset_that_shows_the_node_whatever_the_geometry() {
    for max_lanes in 1..=12 {
        for terminal_width in 0..=60 {
            let graph_width = 2 * max_lanes - 1;
            let window = graph_width.min(terminal_width / 4);
            let max = max_offset(max_lanes, terminal_width);

            for node_column in 0..max_lanes {
                let cell = 2 * node_column;
                let revealable: Vec<usize> = (0..=max).filter(|&o| is_revealed(o, cell, graph_width, window)).collect();
                if revealable.is_empty() {
                    continue;
                }

                for offset in 0..=max + 2 {
                    let closest = revealable.iter().copied().min_by_key(|&o| o.abs_diff(offset)).unwrap();
                    let got = reveal_offset(offset, node_column, max_lanes, terminal_width);

                    assert_eq!(
                        got.abs_diff(offset),
                        closest.abs_diff(offset),
                        "{max_lanes} {terminal_width} {node_column} {offset}"
                    );
                    assert!(revealable.contains(&got), "{max_lanes} {terminal_width} {node_column} {offset}: {got}");
                }
            }
        }
    }
}
