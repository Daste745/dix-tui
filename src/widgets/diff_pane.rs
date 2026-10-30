use std::marker::PhantomData;

use ansi_to_tui::IntoText;
use misstep::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Styled},
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};

use crate::diff::Diff;

#[derive(Debug, Clone, Default)]
pub struct DiffPane<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> StatefulWidget for DiffPane<'a> {
    type State = DiffPaneState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Some(diff) = state.diff else { return };

        let Ok(stats) = stats_text(diff) else { return };

        let [main, bottom] = area.layout(&Layout::vertical([
            Constraint::Fill(1),
            // NOTE: +1 height for the top border
            Constraint::Length(stats.height() as u16 + 1),
        ]));

        render_report(main, buf, diff, &mut state.scroll_state);
        render_stats(bottom, buf, stats);
    }
}

fn stats_text(diff: &Diff) -> Result<Text<'_>> {
    Ok(diff.path_stats.into_text()? + diff.size_diff.into_text()?)
}

fn render_report(area: Rect, buf: &mut Buffer, diff: &Diff, scrollbar_state: &mut ScrollbarState) {
    let [content, scrollbar_area] = area.layout(&Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(1),
    ]));

    let Ok(diff_text) = diff.formatted_report.into_text() else {
        return;
    };
    let paragraph = Paragraph::new(diff_text).scroll((scrollbar_state.get_position() as u16, 0u16));
    paragraph.render(content, buf);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    scrollbar.render(scrollbar_area, buf, scrollbar_state);

    // TODO)) Horizontal scroll for big diffs
    //        This will need some layout adjustments,
    //        Because we handle left/right as changing panes.
}

fn render_stats(area: Rect, buf: &mut Buffer, stats: Text) {
    let block = Block::new()
        .borders(Borders::TOP)
        .merge_borders(MergeStrategy::Exact)
        .border_type(BorderType::Plain)
        .border_style(Color::DarkGray)
        .title("Stats".set_style(Color::White));
    let inner_area = block.inner(area);

    block.render(area, buf);
    stats.render(inner_area, buf);
}

#[derive(Debug, Clone, Copy)]
pub struct DiffPaneState<'a> {
    /// Scrollbar widget state + content scroll offset
    pub scroll_state: ScrollbarState,
    /// Diff of selected profiles
    diff: Option<&'a Diff>,
}

impl<'a> DiffPaneState<'a> {
    pub fn new() -> Self {
        DiffPaneState {
            scroll_state: ScrollbarState::default(),
            diff: None,
        }
    }

    pub fn with_diff(mut self, diff: Option<&'a Diff>) -> Self {
        self.diff = diff;
        self
    }
}
