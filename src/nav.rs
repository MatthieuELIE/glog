use ratatui::{crossterm::event::KeyModifiers, widgets::ListState};

pub fn press(state: &mut ListState, key: char, len: usize) {
    let selected = state.selected().unwrap_or(0);
    if key == 'j' {
        state.select(Some((selected + 1).min(len.saturating_sub(1))));
    }

    if key == 'k' {
        state.select(Some(selected.saturating_sub(1)));
    }
}

pub fn quits(key: char, modifiers: KeyModifiers) -> bool {
    key == 'q' || (key == 'c' && modifiers.contains(KeyModifiers::CONTROL))
}

#[cfg(test)]
mod tests;
