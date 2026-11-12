use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    draw::reveal_offset,
    history::History,
    search::{Search, next_after, scan},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct View {
    pub selected: usize,
    pub top: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    pub view: View,
    pub input: Option<String>,
    pub search: Option<Search>,
}

#[derive(Debug, PartialEq)]
pub enum Step {
    Continue(Session),
    Quit,
}

pub enum Loading {
    Reading,
    Done,
    Failed(String),
}

pub fn list_height(terminal_height: usize) -> usize {
    terminal_height.saturating_sub(1)
}

pub fn page(terminal_height: usize) -> usize {
    (list_height(terminal_height) / 2).max(1)
}

pub fn press(view: View, key: char, modifiers: KeyModifiers, len: usize, page: usize) -> View {
    let last = len.saturating_sub(1);

    if modifiers.contains(KeyModifiers::CONTROL) {
        if key == 'd' {
            return View {
                selected: (view.selected + page).min(last),
                ..view
            };
        }

        if key == 'u' {
            return View {
                selected: view.selected.saturating_sub(page),
                ..view
            };
        }

        return view;
    }

    if key == 'j' {
        return View {
            selected: (view.selected + 1).min(last),
            ..view
        };
    }

    if key == 'k' {
        return View {
            selected: view.selected.saturating_sub(1),
            ..view
        };
    }

    if key == 'g' {
        return View { selected: 0, ..view };
    }

    if key == 'G' {
        return View { selected: last, ..view };
    }

    if key == 'l' {
        return View {
            offset: view.offset + 1,
            ..view
        };
    }

    if key == 'h' {
        return View {
            offset: view.offset.saturating_sub(1),
            ..view
        };
    }

    view
}

pub fn fit(view: View, len: usize, height: usize, max_offset: usize) -> View {
    let selected = view.selected.min(len.saturating_sub(1));
    let top = view.top.min(selected).max((selected + 1).saturating_sub(height));

    View {
        selected,
        top,
        offset: view.offset.min(max_offset),
    }
}

pub fn follow(before: View, after: View, node_offset: usize) -> View {
    if after.selected == before.selected {
        return after;
    }

    View {
        offset: node_offset,
        ..after
    }
}

pub fn quits(key: char, modifiers: KeyModifiers) -> bool {
    key == 'q' || (key == 'c' && modifiers.contains(KeyModifiers::CONTROL))
}

fn settle(before: View, after: View, history: &History, terminal_width: usize) -> View {
    let node_column = history.lane_row(after.selected).node_column as usize;
    let node_offset = reveal_offset(after.offset, node_column, history.max_lanes(), terminal_width);

    follow(before, after, node_offset)
}

fn submit(session: Session, input: String, history: &History, terminal_width: usize) -> Session {
    if input.is_empty() {
        return Session {
            input: Some(input),
            ..session
        };
    }

    let found = scan(history, &input);
    let selected = next_after(&found.matches, session.view.selected).unwrap_or(session.view.selected);
    let after = View { selected, ..session.view };

    Session {
        view: settle(session.view, after, history, terminal_width),
        input: None,
        search: Some(found),
    }
}

fn edit(input: String, key: KeyEvent) -> Option<String> {
    if key.code == KeyCode::Esc {
        return None;
    }

    if key.code == KeyCode::Backspace {
        let mut kept = input.chars();

        return kept.next_back().map(|_| kept.collect());
    }

    match key.code {
        KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => Some(format!("{input}{character}")),
        _ => Some(input),
    }
}

fn type_in(session: Session, input: String, key: KeyEvent, history: &History, terminal_width: usize) -> Step {
    if key.code == KeyCode::Enter {
        return Step::Continue(submit(session, input, history, terminal_width));
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Step::Quit;
    }

    Step::Continue(Session {
        input: edit(input, key),
        ..session
    })
}

pub fn advance(mut session: Session, key: KeyEvent, history: &History, terminal_width: usize, terminal_height: usize) -> Step {
    if let Some(input) = session.input.take() {
        return type_in(session, input, key, history, terminal_width);
    }

    let KeyCode::Char(character) = key.code else {
        return Step::Continue(session);
    };

    if quits(character, key.modifiers) {
        return Step::Quit;
    }

    if character == '/' && !key.modifiers.contains(KeyModifiers::CONTROL) {
        return Step::Continue(Session {
            input: Some(String::new()),
            ..session
        });
    }

    let after = press(session.view, character, key.modifiers, history.len(), page(terminal_height));

    Step::Continue(Session {
        view: settle(session.view, after, history, terminal_width),
        ..session
    })
}

#[cfg(test)]
mod tests;
