use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, style::Color, widgets::ListState};

use crate::{draw::render, nav, parse::parse};

const SEP: &str = "\x1f";

fn line(hash: &str, parents: &str, refs: &str, date: &str, author: &str, subject: &str) -> String {
    [hash, parents, refs, date, author, subject].join(SEP)
}

fn draw(log: &str, width: u16, height: u16) -> Buffer {
    draw_after(log, width, height, "")
}

fn draw_after(log: &str, width: u16, height: u16, keys: &str) -> Buffer {
    let commits = parse(log);
    let mut state = ListState::default().with_selected(Some(0));
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, &commits, &mut state)).unwrap();
    for key in keys.chars() {
        nav::press(&mut state, key, commits.len());
        terminal.draw(|frame| render(frame, &commits, &mut state)).unwrap();
    }

    terminal.backend().buffer().clone()
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol().to_string())
        .collect()
}

#[test]
fn shows_one_row_per_commit_newest_first() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "", "1 day ago", "Ann", "second"),
        line("aaaaaaaa", "", "", "2 days ago", "Ann", "first"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 0).contains("second"));
    assert!(row_text(&buffer, 1).contains("first"));
    assert_eq!(row_text(&buffer, 2).trim(), "");
}

fn two_commits() -> String {
    [
        line("1234567890abcdef", "fedcba0987654321", "", "3 days ago", "Ann", "short subject"),
        line("fedcba0987654321", "", "", "2 weeks ago", "Bartholomew", "older"),
    ]
    .join("\n")
}

#[test]
fn aligns_graph_hash_date_author_and_message_columns() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(row_text(&buffer, 0).trim_end(), "● 1234567 3 days ago  Ann         short subject");
    assert_eq!(row_text(&buffer, 1).trim_end(), "● fedcba0 2 weeks ago Bartholomew older");
}

#[test]
fn draws_head_as_a_hollow_node_and_other_commits_as_filled_nodes() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "origin/main", "1 day ago", "Ann", "second"),
        line("aaaaaaaa", "", "HEAD -> main, tag: v1", "2 days ago", "Ann", "first"),
        line("cccccccc", "", "HEAD", "3 days ago", "Ann", "detached"),
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
        line("1234567890abcdef", "", "", "3 days ago", "山田", "Wide"),
        line("fedcba0987654321", "", "", "3 days ago", "Ann", "Narrow"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    let column_of = |y: u16, symbol: &str| (0..40).find(|&x| buffer[(x, y)].symbol() == symbol);

    assert_eq!(column_of(0, "W"), column_of(1, "N"));
    assert!(column_of(0, "W").is_some());
}

#[test]
fn truncates_the_message_first_in_a_narrow_terminal() {
    let buffer = draw(&two_commits(), 38, 2);

    assert_eq!(row_text(&buffer, 0), "● 1234567 3 days ago  Ann         sho…");
}

#[test]
fn colours_hash_in_overlay_and_text_in_text_and_subtext() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(buffer[(2, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
    assert_eq!(buffer[(10, 0)].fg, Color::Rgb(0xa6, 0xad, 0xc8));
    assert_eq!(buffer[(34, 0)].fg, Color::Rgb(0xcd, 0xd6, 0xf4));
    assert_eq!(buffer[(34, 1)].bg, Color::Reset);
}

#[test]
fn colours_author_in_lavender_apart_from_the_date() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(buffer[(22, 0)].fg, Color::Rgb(0xb4, 0xbe, 0xfe));
    assert_eq!(buffer[(22, 1)].fg, Color::Rgb(0xb4, 0xbe, 0xfe));
}

const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);

fn five_commits() -> String {
    ["e", "d", "c", "b", "a"]
        .map(|name| line(&name.repeat(8), "", "", "1 day ago", "Ann", &format!("commit {name}")))
        .join("\n")
}

fn highlighted_rows(buffer: &Buffer) -> Vec<u16> {
    (0..buffer.area.height)
        .filter(|&y| buffer[(0, y)].bg == SURFACE0)
        .collect()
}

#[test]
fn highlights_the_first_row_in_surface0_at_start() {
    let buffer = draw(&five_commits(), 40, 5);

    assert_eq!(highlighted_rows(&buffer), vec![0]);
    assert_eq!(buffer[(39, 0)].bg, SURFACE0);
    assert_eq!(buffer[(2, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
}

#[test]
fn j_moves_the_highlight_down_one_commit() {
    let buffer = draw_after(&five_commits(), 40, 5, "jj");

    assert_eq!(highlighted_rows(&buffer), vec![2]);
}

#[test]
fn j_scrolls_the_view_to_keep_the_selection_visible() {
    let buffer = draw_after(&five_commits(), 40, 3, "jjjj");

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
    let buffer = draw_after(&five_commits(), 40, 3, "jjjjkkk");

    assert!(row_text(&buffer, 0).contains("commit d"));
    assert_eq!(highlighted_rows(&buffer), vec![0]);
}

#[test]
fn k_on_the_first_commit_does_nothing() {
    let at_first = draw_after(&five_commits(), 40, 3, "");
    let before_first = draw_after(&five_commits(), 40, 3, "kk");

    assert_eq!(at_first, before_first);
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
        line("ccccccc", "bbbbbbb", "", "1 day ago", "Ann", "c"),
        line("bbbbbbb", "aaaaaaa", "", "1 day ago", "Ann", "b"),
        line("sssssss", "aaaaaaa", "", "1 day ago", "Ann", "s"),
        line("aaaaaaa", "", "", "1 day ago", "Ann", "a"),
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
        line("mmmmmmm", "bbbbbbb sssssss", "", "1 day ago", "Ann", "m"),
        line("bbbbbbb", "aaaaaaa", "", "1 day ago", "Ann", "b"),
        line("sssssss", "aaaaaaa", "", "1 day ago", "Ann", "s"),
        line("aaaaaaa", "", "", "1 day ago", "Ann", "a"),
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
        line("mmmmmmm", "bbbbbbb sssssss ttttttt", "", "1 day ago", "Ann", "m"),
        line("bbbbbbb", "", "", "1 day ago", "Ann", "b"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    assert_eq!(graph(&buffer, 0, 5), "●─╮─╮");
    assert_eq!(graph(&buffer, 1, 5), "● │ │");
}

fn tips(count: usize) -> String {
    (0..count)
        .map(|n| line(&format!("tip{n:04}"), "", "", "1 day ago", "Ann", "t"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn open_lanes_log(count: usize) -> String {
    let parents: Vec<String> = (0..count).map(|n| format!("par{n:04}")).collect();

    line("mmmmmmm", &parents.join(" "), "", "1 day ago", "Ann", "m")
}

#[test]
fn colours_each_lane_in_the_mocha_cycle() {
    let buffer = draw(&open_lanes_log(8), 40, 1);
    let colours: Vec<Color> = (0..8).map(|column| buffer[(2 * column, 0)].fg).collect();

    assert_eq!(colours, [MAUVE, BLUE, GREEN, PEACH, PINK, TEAL, YELLOW, RED]);
}

#[test]
fn restarts_the_cycle_after_the_ninth_lane() {
    let buffer = draw(&open_lanes_log(9), 40, 1);

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
        line(&full("1111111"), &full("2222222"), "", "1 day ago", "Ann", "tip"),
        line(&full("2222222"), "", "", "1 day ago", "Ann", "root"),
        line(&full("3333333"), "", "", "1 day ago", "Ann", "other"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 1).starts_with("● 2222222"));
}

#[test]
fn a_new_lane_takes_the_leftmost_free_column() {
    let log = [
        line("xxxxxxx", "x000000", "", "1 day ago", "Ann", "x"),
        line("yyyyyyy", "", "", "1 day ago", "Ann", "y"),
        line("wwwwwww", "w000000", "", "1 day ago", "Ann", "w"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert_eq!(graph(&buffer, 1, 3), "│ ●");
    assert_eq!(graph(&buffer, 2, 3), "│ ●");
}

fn badge_log(refs: &str) -> String {
    line("aaaaaaa", "", refs, "1 day ago", "Ann", "subject")
}

fn colour_at(buffer: &Buffer, y: u16, text: &str) -> Color {
    let row = row_text(buffer, y);
    let x = row.find(text).unwrap_or_else(|| panic!("{text} not in {row}"));

    buffer[(row[..x].chars().count() as u16, y)].fg
}

#[test]
fn shows_the_refs_of_a_commit_as_badges_before_its_message() {
    let log = badge_log("HEAD -> refs/heads/main, refs/remotes/origin/main, refs/remotes/origin/HEAD, tag: refs/tags/v1.0");

    let buffer = draw(&log, 80, 1);

    assert_eq!(
        row_text(&buffer, 0).trim_end(),
        "○ aaaaaaa 1 day ago Ann [HEAD] [main] [origin/main] [◆ v1.0] subject"
    );
}

#[test]
fn colours_a_local_branch_badge_in_the_lane_colour_of_its_commit() {
    let log = [
        line("ccccccc", "bbbbbbb", "", "1 day ago", "Ann", "c"),
        line("bbbbbbb", "aaaaaaa", "", "1 day ago", "Ann", "b"),
        line("sssssss", "aaaaaaa", "refs/heads/side", "1 day ago", "Ann", "s"),
        line("aaaaaaa", "", "", "1 day ago", "Ann", "a"),
    ]
    .join("\n");

    let buffer = draw(&log, 60, 4);

    assert_eq!(colour_at(&buffer, 2, "[side]"), BLUE);
}

#[test]
fn dims_a_remote_branch_badge() {
    let buffer = draw(&badge_log("refs/remotes/origin/main"), 80, 1);

    assert_eq!(colour_at(&buffer, 0, "[origin/main]"), Color::Rgb(0x6c, 0x70, 0x86));
}

#[test]
fn colours_a_tag_badge_in_yellow() {
    let buffer = draw(&badge_log("tag: refs/tags/v1.0"), 80, 1);

    assert_eq!(colour_at(&buffer, 0, "[◆ v1.0]"), YELLOW);
}

#[test]
fn highlights_head_on_a_branch_and_detached() {
    let on_branch = draw(&badge_log("HEAD -> refs/heads/main"), 80, 1);
    let detached = draw(&badge_log("HEAD"), 80, 1);

    for buffer in [&on_branch, &detached] {
        let x = row_text(buffer, 0).find("[HEAD]").unwrap() as u16;
        assert!(buffer[(x, 0)].modifier.contains(ratatui::style::Modifier::BOLD));
        assert_eq!(buffer[(x, 0)].fg, Color::Rgb(0xcd, 0xd6, 0xf4));
    }
    assert!(!row_text(&detached, 0).contains("[main]"));
}

#[test]
fn gives_origin_head_no_badge() {
    let buffer = draw(&badge_log("refs/remotes/origin/HEAD"), 80, 1);

    assert!(!row_text(&buffer, 0).contains('['));
}

#[test]
fn leaves_the_terminal_background_unpainted_under_badges() {
    let log = [
        line("bbbbbbb", "aaaaaaa", "", "1 day ago", "Ann", "child"),
        badge_log("HEAD -> refs/heads/main, tag: refs/tags/v1.0"),
    ]
    .join("\n");
    let buffer = draw(&log, 80, 2);

    assert!((0..80).all(|x| buffer[(x, 1)].bg == Color::Reset));
}

#[test]
fn truncates_the_message_after_the_badges_in_a_narrow_terminal() {
    let buffer = draw(&badge_log("refs/heads/main"), 36, 1);

    assert_eq!(row_text(&buffer, 0), "● aaaaaaa 1 day ago Ann [main] subj…");
}

#[test]
fn a_lane_passing_under_a_junction_keeps_its_vertical_line() {
    let log = [
        line("xxxxxxx", "aaaaaaa", "", "1 day ago", "Ann", "x"),
        line("yyyyyyy", "qqqqqqq", "", "1 day ago", "Ann", "y"),
        line("sssssss", "aaaaaaa", "", "1 day ago", "Ann", "s"),
        line("aaaaaaa", "", "", "1 day ago", "Ann", "a"),
        line("qqqqqqq", "", "", "1 day ago", "Ann", "q"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 5);

    assert_eq!(graph(&buffer, 3, 5), "●─│─╯");
    assert_eq!(buffer[(2, 3)].fg, BLUE);
}

#[test]
fn a_merge_parent_takes_the_column_freed_by_a_lane_ending_on_the_same_row() {
    let log = [
        line("xxxxxxx", "mmmmmmm", "", "1 day ago", "Ann", "x"),
        line("sssssss", "mmmmmmm", "", "1 day ago", "Ann", "s"),
        line("mmmmmmm", "bbbbbbb ppppppp", "", "1 day ago", "Ann", "m"),
        line("bbbbbbb", "", "", "1 day ago", "Ann", "b"),
        line("ppppppp", "", "", "1 day ago", "Ann", "p"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 5);

    assert!(row_text(&buffer, 2).starts_with("●─┤ mmmmmmm"));
    assert_eq!(buffer[(2, 2)].fg, GREEN);
    assert_eq!(graph(&buffer, 3, 3), "● │");
}

#[test]
fn orders_badges_head_then_local_then_remote_then_tags_keeping_git_order_within_a_kind() {
    let log = badge_log("tag: refs/tags/v1, refs/remotes/origin/feature/a, refs/heads/zeta, refs/heads/feature/a, HEAD");

    let buffer = draw(&log, 100, 1);

    assert_eq!(
        row_text(&buffer, 0).trim_end(),
        "○ aaaaaaa 1 day ago Ann [HEAD] [zeta] [feature/a] [origin/feature/a] [◆ v1] subject"
    );
}
