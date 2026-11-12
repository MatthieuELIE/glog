use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    history::History,
    nav::{Step, View, advance, fit, follow, list_height, page, press, quits},
    parse::parse,
};

#[test]
fn q_quits() {
    assert!(quits('q', KeyModifiers::NONE));
}

#[test]
fn ctrl_c_quits() {
    assert!(quits('c', KeyModifiers::CONTROL));
}

#[test]
fn c_without_ctrl_does_not_quit() {
    assert!(!quits('c', KeyModifiers::NONE));
}

#[test]
fn ctrl_with_another_letter_does_not_quit() {
    assert!(!quits('j', KeyModifiers::CONTROL));
}

const NONE: KeyModifiers = KeyModifiers::NONE;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;

fn view(selected: usize, top: usize, offset: usize) -> View {
    View { selected, top, offset }
}

#[test]
fn fit_scrolls_down_just_enough_for_a_selection_below_the_last_visible_row() {
    assert_eq!(fit(view(5, 0, 0), 10, 3, 0), view(5, 3, 0));
}

#[test]
fn fit_scrolls_up_to_a_selection_above_the_first_visible_row() {
    assert_eq!(fit(view(1, 4, 0), 10, 3, 0), view(1, 1, 0));
}

#[test]
fn fit_keeps_the_top_when_the_selection_is_visible() {
    assert_eq!(fit(view(4, 3, 0), 10, 3, 0), view(4, 3, 0));
}

#[test]
fn fit_scrolls_when_a_resize_shrinks_the_height_below_the_selection() {
    assert_eq!(fit(view(5, 3, 0), 10, 6, 0), view(5, 3, 0));
    assert_eq!(fit(view(5, 3, 0), 10, 2, 0), view(5, 4, 0));
}

#[test]
fn fit_brings_the_selection_and_the_offset_back_within_their_bounds() {
    assert_eq!(fit(view(9, 0, 7), 4, 10, 2), view(3, 0, 2));
}

#[test]
fn press_moves_the_selection_and_the_offset_within_their_edges() {
    assert_eq!(press(view(0, 0, 0), 'k', NONE, 3, 2), view(0, 0, 0));
    assert_eq!(press(view(0, 0, 0), 'j', NONE, 3, 2), view(1, 0, 0));
    assert_eq!(press(view(2, 0, 0), 'j', NONE, 3, 2), view(2, 0, 0));
    assert_eq!(press(view(0, 0, 0), 'h', NONE, 3, 2), view(0, 0, 0));
    assert_eq!(press(view(0, 0, 4), 'l', NONE, 3, 2), view(0, 0, 5));
    assert_eq!(press(view(0, 0, 4), 'h', NONE, 3, 2), view(0, 0, 3));
}

#[test]
fn g_and_capital_g_jump_to_the_first_and_the_last_commit_and_leave_the_offset() {
    assert_eq!(press(view(4, 2, 3), 'g', NONE, 10, 2), view(0, 2, 3));
    assert_eq!(press(view(4, 2, 3), 'G', NONE, 10, 2), view(9, 2, 3));
    assert_eq!(press(view(0, 0, 0), 'G', NONE, 0, 2), view(0, 0, 0));
}

#[test]
fn ctrl_d_and_ctrl_u_move_a_page_within_the_edges_and_leave_the_offset() {
    assert_eq!(press(view(1, 0, 3), 'd', CTRL, 10, 4), view(5, 0, 3));
    assert_eq!(press(view(8, 0, 3), 'd', CTRL, 10, 4), view(9, 0, 3));
    assert_eq!(press(view(5, 0, 3), 'u', CTRL, 10, 4), view(1, 0, 3));
    assert_eq!(press(view(2, 0, 3), 'u', CTRL, 10, 4), view(0, 0, 3));
}

#[test]
fn d_and_u_need_ctrl_and_the_other_keys_refuse_it() {
    assert_eq!(press(view(1, 0, 0), 'd', NONE, 10, 4), view(1, 0, 0));
    assert_eq!(press(view(5, 0, 0), 'u', NONE, 10, 4), view(5, 0, 0));
    for key in ['j', 'k', 'h', 'l', 'g', 'G'] {
        assert_eq!(press(view(5, 0, 5), key, CTRL, 10, 4), view(5, 0, 5));
    }
}

#[test]
fn the_list_always_loses_its_last_row_to_the_status_line() {
    assert_eq!(list_height(50), 49);
    assert_eq!(list_height(0), 0);
}

#[test]
fn a_page_is_half_the_list_height_and_at_least_one_row() {
    assert_eq!(page(50), 24);
    assert_eq!(page(51), 25);
    assert_eq!(page(1), 1);
    assert_eq!(page(0), 1);
}

#[test]
fn follow_takes_the_node_offset_when_the_selection_changed() {
    let before = View {
        selected: 1,
        top: 0,
        offset: 2,
    };
    let after = View {
        selected: 2,
        top: 0,
        offset: 2,
    };

    assert_eq!(follow(before, after, 7), View { offset: 7, ..after });
}

#[test]
fn follow_keeps_the_view_when_the_selection_did_not_change() {
    let before = View {
        selected: 1,
        top: 0,
        offset: 2,
    };
    let after = View { offset: 3, ..before };

    assert_eq!(follow(before, after, 7), after);
}

fn history_of(count: usize) -> History {
    let mut history = History::default();
    for index in (0..count).rev() {
        let parents = if index == 0 { String::new() } else { format!("{:08x}", index - 1) };
        let log = [
            format!("{index:08x}"),
            parents,
            String::new(),
            "1913600".to_string(),
            "Ann".to_string(),
            "subject".to_string(),
        ]
        .join("\x1f");
        parse(&log).into_iter().for_each(|commit| history.push(commit));
    }

    history
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn advance_moves_the_selection_and_follows_the_node() {
    let history = history_of(5);

    let step = advance(View::default(), key(KeyCode::Char('j'), NONE), &history, 60, 10);

    assert_eq!(step, Step::Continue(view(1, 0, 0)));
}

#[test]
fn advance_quits_on_q_and_ctrl_c() {
    let history = history_of(5);

    assert_eq!(advance(View::default(), key(KeyCode::Char('q'), NONE), &history, 60, 10), Step::Quit);
    assert_eq!(advance(View::default(), key(KeyCode::Char('c'), CTRL), &history, 60, 10), Step::Quit);
}

#[test]
fn advance_ignores_a_key_that_is_not_a_character() {
    let history = history_of(5);

    for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Backspace] {
        assert_eq!(advance(view(2, 0, 0), key(code, NONE), &history, 60, 10), Step::Continue(view(2, 0, 0)));
    }
}
