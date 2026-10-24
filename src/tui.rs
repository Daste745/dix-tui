use std::collections::HashMap;

use ansi_to_tui::IntoText;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Margin, Rect, Spacing},
    style::{Color, Modifier, Styled},
    symbols::merge::MergeStrategy,
    widgets::{
        Block, BorderType, List, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::{diff::display_diff, profile::Profile};

pub fn run(mut terminal: DefaultTerminal, state: &mut TuiState) -> Result<()> {
    state.initialize()?;

    loop {
        // TODO)) Friendlier error handling
        terminal.draw(|frame| render(frame, state).unwrap())?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => match state.selected_pane {
                    Pane::OldProfiles | Pane::NewProfiles => state.handle_next_profile()?,
                    Pane::Diff => state.diff_scrollbar.next(),
                },
                KeyCode::Char('k') | KeyCode::Up => match state.selected_pane {
                    Pane::OldProfiles | Pane::NewProfiles => state.handle_prev_profile()?,
                    Pane::Diff => state.diff_scrollbar.prev(),
                },
                KeyCode::Char('g') => match state.selected_pane {
                    Pane::OldProfiles | Pane::NewProfiles => state.handle_first_profile()?,
                    Pane::Diff => state.diff_scrollbar.first(),
                },
                KeyCode::Char('G') => match state.selected_pane {
                    Pane::OldProfiles | Pane::NewProfiles => state.handle_last_profile()?,
                    Pane::Diff => state.diff_scrollbar.last(),
                },
                KeyCode::Char('h') | KeyCode::Left => state.handle_prev_pane(),
                KeyCode::Char('l') | KeyCode::Right => state.handle_next_pane(),
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Pane {
    OldProfiles,
    NewProfiles,
    Diff,
}

pub struct TuiState {
    pub profiles: Vec<Profile>,
    selected_pane: Pane,
    old_profile_list: ListState,
    new_profile_list: ListState,
    diff_scrollbar: ScrollbarState,
    diff_content: Option<DiffContent>,
}

impl TuiState {
    pub fn new(profiles: Vec<Profile>) -> Self {
        assert!(!profiles.is_empty(), "At least one profile is required");

        let last_profile_idx = profiles.len() - 1;
        // Select the same profile if there's only one
        let prev_profile_idx = if profiles.len() > 1 {
            last_profile_idx - 1
        } else {
            last_profile_idx
        };

        Self {
            profiles,
            selected_pane: Pane::OldProfiles,
            old_profile_list: ListState::default().with_selected(Some(prev_profile_idx)),
            new_profile_list: ListState::default().with_selected(Some(last_profile_idx)),
            diff_scrollbar: ScrollbarState::default(),
            diff_content: None,
        }
    }

    /// Initializes the TUI state before the first render.
    fn initialize(&mut self) -> Result<()> {
        self.update_diff_content()?;
        Ok(())
    }

    /// Returns a mutable reference to the selected profile list, if any.
    fn selected_profile_list(&mut self) -> Option<&mut ListState> {
        match self.selected_pane {
            Pane::OldProfiles => Some(&mut self.old_profile_list),
            Pane::NewProfiles => Some(&mut self.new_profile_list),
            _ => None,
        }
    }

    fn handle_next_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            Pane::OldProfiles => Pane::NewProfiles,
            Pane::NewProfiles => Pane::Diff,
            Pane::Diff => Pane::OldProfiles,
        };
    }

    fn handle_prev_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            Pane::OldProfiles => Pane::Diff,
            Pane::NewProfiles => Pane::OldProfiles,
            Pane::Diff => Pane::NewProfiles,
        };
    }

    fn handle_next_profile(&mut self) -> Result<()> {
        self.change_profile_selection(ListState::select_next)
    }

    fn handle_prev_profile(&mut self) -> Result<()> {
        self.change_profile_selection(ListState::select_previous)
    }

    fn handle_first_profile(&mut self) -> Result<()> {
        self.change_profile_selection(ListState::select_first)
    }

    fn handle_last_profile(&mut self) -> Result<()> {
        self.change_profile_selection(ListState::select_last)
    }

    /// Applies `f` to the selected profile list, clamps the selection
    /// to the last index and updates the diff content.
    fn change_profile_selection(&mut self, f: impl FnOnce(&mut ListState)) -> Result<()> {
        let last_idx = self.profiles.len().saturating_sub(1);
        if let Some(list) = self.selected_profile_list() {
            f(list);
            if let Some(idx) = list.selected() {
                // Clamp list selection to the last index
                list.select(Some(idx.min(last_idx)));
            }
            self.update_diff_content()?;
        }
        Ok(())
    }

    /// Updates the diff content state based on the selected profiles.
    fn update_diff_content(&mut self) -> Result<()> {
        self.diff_scrollbar.first();
        self.diff_content = self.new_diff_content()?;
        if let Some(c) = &self.diff_content {
            self.diff_scrollbar = self.diff_scrollbar.content_length(c.lines);
        }
        Ok(())
    }

    /// Builds a new diff content from selected profiles.
    fn new_diff_content(&self) -> Result<Option<DiffContent>> {
        let old = self
            .old_profile_list
            .selected()
            .and_then(|i| self.profiles.get(i));
        let new = self
            .new_profile_list
            .selected()
            .and_then(|i| self.profiles.get(i));

        let result = match (old, new) {
            (Some(old), Some(new)) => {
                let mut out = String::new();
                display_diff(&mut out, &old.path, &new.path)?;
                let lines = out.lines().count();
                Some(DiffContent { text: out, lines })
            }
            _ => None,
        };
        Ok(result)
    }
}

struct DiffContent {
    text: String,
    lines: usize,
}

fn render(frame: &mut Frame, state: &mut TuiState) -> Result<()> {
    let [left, middle, right] = frame.area().layout(
        &Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(6),
        ])
        .spacing(Spacing::Overlap(1)),
    );

    let mut panes = HashMap::new();
    panes.insert(Pane::OldProfiles, (left, "Old"));
    panes.insert(Pane::NewProfiles, (middle, "New"));
    panes.insert(Pane::Diff, (right, "Diff"));

    // Render all panes except the selected one first
    for (&pane, &(area, title)) in &panes {
        if pane != state.selected_pane {
            let block = Block::bordered()
                .merge_borders(MergeStrategy::Exact)
                .border_type(BorderType::Plain)
                .border_style(Color::DarkGray)
                .title(title.set_style(Color::White));
            frame.render_widget(&block, area);
            render_pane_content(frame, block.inner(area), pane, state)?;
        }
    }

    // Render the selected pane last (so it appears on top) with a thick border
    if let Some(&(area, title)) = panes.get(&state.selected_pane) {
        let block = Block::bordered()
            .merge_borders(MergeStrategy::Exact)
            .border_type(BorderType::Thick)
            .border_style(Color::Gray)
            .title(title.set_style(Color::White));
        frame.render_widget(&block, area);
        render_pane_content(frame, block.inner(area), state.selected_pane, state)?;
    }

    Ok(())
}

fn render_pane_content(
    frame: &mut Frame,
    area: Rect,
    pane: Pane,
    state: &mut TuiState,
) -> Result<()> {
    match pane {
        Pane::OldProfiles => render_list(frame, area, &state.profiles, &mut state.old_profile_list),
        Pane::NewProfiles => render_list(frame, area, &state.profiles, &mut state.new_profile_list),
        Pane::Diff => render_diff_pane(frame, area, state)?,
    }

    Ok(())
}

fn render_list(frame: &mut Frame, area: Rect, items: &[Profile], state: &mut ListState) {
    // TODO)) Nicer formatting of profile names
    let list = List::new(items.iter().map(|it| it.to_string())).highlight_style(Modifier::REVERSED);
    frame.render_stateful_widget(list, area, state);
}

fn render_diff_pane(frame: &mut Frame, area: Rect, state: &mut TuiState) -> Result<()> {
    // Diff content
    if let Some(c) = &state.diff_content {
        let paragraph = Paragraph::new(c.text.into_text()?)
            .scroll((state.diff_scrollbar.get_position() as u16, 0u16));
        frame.render_widget(paragraph, area);
    }

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut state.diff_scrollbar,
    );
    // TODO)) Horizontal scroll for big diffs
    //        This will need some layout adjustments,
    //        Because we handle left/right as changing panes.

    Ok(())
}
