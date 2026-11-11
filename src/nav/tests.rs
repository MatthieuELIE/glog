use ratatui::crossterm::event::KeyModifiers;

use crate::nav::{Loading, View, fit, list_height, press, quits};

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
    assert_eq!(press(view(0, 0, 0), 'k', 3), view(0, 0, 0));
    assert_eq!(press(view(0, 0, 0), 'j', 3), view(1, 0, 0));
    assert_eq!(press(view(2, 0, 0), 'j', 3), view(2, 0, 0));
    assert_eq!(press(view(0, 0, 0), 'h', 3), view(0, 0, 0));
    assert_eq!(press(view(0, 0, 4), 'l', 3), view(0, 0, 5));
    assert_eq!(press(view(0, 0, 4), 'h', 3), view(0, 0, 3));
}

#[test]
fn the_list_loses_its_last_row_to_the_status_line_unless_loading_is_done() {
    assert_eq!(list_height(50, &Loading::Reading), 49);
    assert_eq!(list_height(50, &Loading::Failed(String::new())), 49);
    assert_eq!(list_height(50, &Loading::Done), 50);
    assert_eq!(list_height(0, &Loading::Reading), 0);
}
