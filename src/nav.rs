use ratatui::crossterm::event::KeyModifiers;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct View {
    pub selected: usize,
    pub top: usize,
    pub offset: usize,
}

pub enum Loading {
    Reading,
    Done,
    Failed(String),
}

pub fn list_height(terminal_height: usize, loading: &Loading) -> usize {
    if matches!(loading, Loading::Done) {
        return terminal_height;
    }

    terminal_height.saturating_sub(1)
}

pub fn page(terminal_height: usize, loading: &Loading) -> usize {
    (list_height(terminal_height, loading) / 2).max(1)
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

#[cfg(test)]
mod tests;
