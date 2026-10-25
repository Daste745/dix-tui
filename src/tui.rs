use std::collections::HashMap;

use ansi_to_tui::IntoText;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Margin, Rect, Spacing},
    style::{Color, Modifier, Styled, Stylize},
    symbols::merge::MergeStrategy,
    text::{Line, Span},
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

        // TODO)) Handle errors from key press events
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => match state.selected_pane {
                    Pane::Profiles => state.handle_next_profile()?,
                    Pane::Diff => state.diff_scrollbar.next(),
                },
                KeyCode::Char('k') | KeyCode::Up => match state.selected_pane {
                    Pane::Profiles => state.handle_prev_profile()?,
                    Pane::Diff => state.diff_scrollbar.prev(),
                },
                KeyCode::Char('g') => match state.selected_pane {
                    Pane::Profiles => state.handle_first_profile()?,
                    Pane::Diff => state.diff_scrollbar.first(),
                },
                KeyCode::Char('G') => match state.selected_pane {
                    Pane::Profiles => state.handle_last_profile()?,
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
    Profiles,
    Diff,
}

pub struct TuiState {
    profiles: Vec<Profile>,
    selected_pane: Pane,
    profile_list: ListState,
    diff_scrollbar: ScrollbarState,
    diff_content: Option<DiffContent>,
}

impl TuiState {
    pub fn new(profiles: Vec<Profile>) -> Self {
        let last_profile_idx = profiles.len().saturating_sub(1);

        Self {
            profiles,
            selected_pane: Pane::Profiles,
            profile_list: ListState::default().with_selected(Some(last_profile_idx)),
            diff_scrollbar: ScrollbarState::default(),
            diff_content: None,
        }
    }

    /// Initializes the TUI state before the first render.
    fn initialize(&mut self) -> Result<()> {
        self.update_diff_content()?;
        Ok(())
    }

    fn handle_next_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            Pane::Profiles => Pane::Diff,
            Pane::Diff => Pane::Profiles,
        };
    }

    fn handle_prev_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            Pane::Profiles => Pane::Diff,
            Pane::Diff => Pane::Profiles,
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

    /// Applies `f` to the profile list, clamps the selection
    /// to the last index and updates the diff content.
    fn change_profile_selection(&mut self, f: impl FnOnce(&mut ListState)) -> Result<()> {
        let last_idx = self.profiles.len().saturating_sub(1);
        f(&mut self.profile_list);
        if let Some(idx) = self.profile_list.selected() {
            // Clamp list selection to the last index
            self.profile_list.select(Some(idx.min(last_idx)));
            self.update_diff_content()?;
        }
        Ok(())
    }

    /// Updates the diff content state based on the selected old/new profiles.
    fn update_diff_content(&mut self) -> Result<()> {
        self.diff_scrollbar.first();
        self.diff_content = self.new_diff_content()?;
        if let Some(c) = &self.diff_content {
            self.diff_scrollbar = self.diff_scrollbar.content_length(c.lines);
        }
        Ok(())
    }

    /// Builds a new diff content from selected old/new profiles.
    fn new_diff_content(&self) -> Result<Option<DiffContent>> {
        let (old, new) = self.selected_profiles();
        let mut out = String::new();
        display_diff(&mut out, &old.path, &new.path)?;
        let lines = out.lines().count();
        Ok(Some(DiffContent { text: out, lines }))
    }

    /// Returns selected old/new profiles.
    ///
    /// If there's only one profile, it is returned as both old and new.
    fn selected_profiles(&self) -> (&Profile, &Profile) {
        let (old_idx, new_idx) = self.selected_profile_indexes();
        let old = self
            .profiles
            .get(old_idx)
            .expect("At least one profile must exist");
        let new = self
            .profiles
            .get(new_idx)
            .expect("At least one profile must exist");
        (old, new)
    }

    /// Returns selected old/new profile indexes.
    ///
    /// If there's only one profile, it is returned as both old and new.
    fn selected_profile_indexes(&self) -> (usize, usize) {
        let new = self.profile_list.selected().unwrap_or(0);
        // TODO)) Allow to manually pin this to an arbitrary index
        let old = new.saturating_sub(1);
        (old, new)
    }
}

struct DiffContent {
    text: String,
    lines: usize,
}

fn render(frame: &mut Frame, state: &mut TuiState) -> Result<()> {
    let [left, right] = frame.area().layout(
        &Layout::horizontal([Constraint::Fill(1), Constraint::Fill(6)])
            .spacing(Spacing::Overlap(1)),
    );

    let mut panes = HashMap::new();
    panes.insert(Pane::Profiles, (left, "Profiles"));
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
        Pane::Profiles => render_profile_list(
            frame,
            area,
            &state.profiles,
            state.selected_profile_indexes(),
            &mut state.profile_list,
        ),
        Pane::Diff => render_diff_pane(frame, area, state)?,
    }

    Ok(())
}

fn render_profile_list(
    frame: &mut Frame,
    area: Rect,
    items: &[Profile],
    selected: (usize, usize),
    state: &mut ListState,
) {
    let (old_idx, new_idx) = selected;
    let list = List::new(items.iter().enumerate().map(|(idx, it)| {
        let prefix = if idx == old_idx && idx == new_idx {
            "=".green()
        } else if idx == old_idx {
            "-".red()
        } else if idx == new_idx {
            "+".green()
        } else {
            Span::raw(" ")
        };
        // TODO)) Nicer formatting of profile names
        Line::from(vec![prefix, " ".into(), it.to_string().into()])
    }))
    .highlight_style(Modifier::BOLD);
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
