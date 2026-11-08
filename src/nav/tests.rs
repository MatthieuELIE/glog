use ratatui::crossterm::event::KeyModifiers;

use crate::nav::quits;

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
