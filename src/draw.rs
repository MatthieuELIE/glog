use ratatui::{Frame, text::Line, widgets::Paragraph};

use crate::parse::Commit;

pub fn render(frame: &mut Frame, commits: &[Commit]) {
    let lines: Vec<Line> = commits
        .iter()
        .map(|commit| Line::raw(commit.subject.clone()))
        .collect();

    frame.render_widget(Paragraph::new(lines), frame.area());
}

#[cfg(test)]
mod tests;
