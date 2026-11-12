use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{draw::reveal_offset, history::History};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct View {
    pub selected: usize,
    pub top: usize,
    pub offset: usize,
}

#[derive(Debug, PartialEq)]
pub enum Step {
    Continue(View),
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

pub fn advance(view: View, key: KeyEvent, history: &History, terminal_width: usize, terminal_height: usize) -> Step {
    let KeyCode::Char(character) = key.code else {
        return Step::Continue(view);
    };

    if quits(character, key.modifiers) {
        return Step::Quit;
    }

    let after = press(view, character, key.modifiers, history.len(), page(terminal_height));
    let node_column = history.lane_row(after.selected).node_column as usize;
    let node_offset = reveal_offset(after.offset, node_column, history.max_lanes(), terminal_width);

    Step::Continue(follow(view, after, node_offset))
}

#[cfg(test)]
mod tests;
