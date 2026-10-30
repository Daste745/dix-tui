use std::collections::HashMap;

use lru::LruCache;
use misstep::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Rect, Spacing},
    style::{Color, Styled},
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{Block, BorderType},
};

use crate::{
    diff::Diff,
    profile::Profile,
    widgets::{DiffPane, DiffPaneState, ProfilesPane, ProfilesPaneState},
};

pub fn run(mut terminal: DefaultTerminal, state: &mut TuiState) -> Result<()> {
    state.initialize()?;

    loop {
        // TODO)) Friendlier error handling
        terminal.draw(|frame| render(frame, state).unwrap())?;

        // TODO)) Handle errors from key press events
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => match state.selected_pane {
                    Pane::Profiles => state.handle_profile_selection(ProfilesPaneState::next)?,
                    Pane::Diff => state.diff_pane.scroll_state.next(),
                },
                KeyCode::Char('k') | KeyCode::Up => match state.selected_pane {
                    Pane::Profiles => state.handle_profile_selection(ProfilesPaneState::prev)?,
                    Pane::Diff => state.diff_pane.scroll_state.prev(),
                },
                KeyCode::Char('g') | KeyCode::Home => match state.selected_pane {
                    Pane::Profiles => state.handle_profile_selection(ProfilesPaneState::first)?,
                    Pane::Diff => state.diff_pane.scroll_state.first(),
                },
                KeyCode::Char('G') | KeyCode::End => match state.selected_pane {
                    Pane::Profiles => state.handle_profile_selection(ProfilesPaneState::last)?,
                    Pane::Diff => state.diff_pane.scroll_state.last(),
                },
                KeyCode::Tab | KeyCode::Char('h') | KeyCode::Left => state.handle_prev_pane(),
                KeyCode::BackTab | KeyCode::Char('l') | KeyCode::Right => state.handle_next_pane(),
                KeyCode::Char(' ') if state.selected_pane == Pane::Profiles => {
                    state.handle_profile_selection(ProfilesPaneState::handle_pin_profile)?;
                }
                KeyCode::Char('`') => state.show_debug = !state.show_debug,
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

pub struct TuiState<'a> {
    profiles: Vec<Profile>,
    selected_pane: Pane,
    show_debug: bool,
    profiles_pane: ProfilesPaneState,
    diff_pane: DiffPaneState<'a>,
    diff_cache: LruCache<(usize, usize), Diff>,
}

impl<'a> TuiState<'a> {
    pub fn new(profiles: Vec<Profile>) -> Self {
        let profile_count = profiles.len();

        Self {
            profiles,
            selected_pane: Pane::Profiles,
            show_debug: false,
            profiles_pane: ProfilesPaneState::new(profile_count),
            diff_pane: DiffPaneState::new(),
            // TODO)) Bound the LRU cache size to some reasonable value
            //        100 diffs is ~4MB of memory
            diff_cache: LruCache::unbounded(),
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

    fn handle_profile_selection(&mut self, f: impl FnOnce(&mut ProfilesPaneState)) -> Result<()> {
        f(&mut self.profiles_pane);
        self.update_diff_content()
    }

    /// Pushes the new diff content into the cache.
    fn update_diff_content(&mut self) -> Result<()> {
        self.diff_pane.scroll_state.first();

        let cache_key = self.profiles_pane.selected();
        if let Some(diff) = self.diff_cache.get(&cache_key) {
            self.diff_pane.scroll_state = self
                .diff_pane
                .scroll_state
                .content_length(diff.formatted_report.lines().count());
            return Ok(());
        }

        let (old, new) = self.selected_profiles();
        let diff = Diff::new(old.to_owned(), new.to_owned())?;
        let content_length = diff.formatted_report.lines().count();
        self.diff_cache.put(cache_key, diff);
        self.diff_pane.scroll_state = self.diff_pane.scroll_state.content_length(content_length);

        Ok(())
    }

    /// Returns selected old/new profiles.
    ///
    /// If there's only one profile, it is returned as both old and new.
    fn selected_profiles(&self) -> (&Profile, &Profile) {
        let (old_idx, new_idx) = self.profiles_pane.selected();
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
}

fn render(frame: &mut Frame, state: &mut TuiState) -> Result<()> {
    let debug_height = if state.show_debug { 1 } else { 0 };
    let [main, debug] =
        Layout::vertical([Constraint::Fill(5), Constraint::Fill(debug_height)]).areas(frame.area());

    let [left, right] = main.layout(
        &Layout::horizontal([Constraint::Fill(1), Constraint::Fill(6)])
            .spacing(Spacing::Overlap(1)),
    );

    let mut panes = HashMap::new();
    panes.insert(Pane::Profiles, left);
    panes.insert(Pane::Diff, right);

    // Render unselected panes first, then the selected one last,
    // so it appears on top with a thick border.
    let mut order: Vec<_> = panes.keys().copied().collect();
    order.sort_by_key(|&pane| pane == state.selected_pane);
    for pane in order {
        let area = panes[&pane];
        let title = pane_title(pane, state);
        let selected = pane == state.selected_pane;
        let block = pane_block(selected).title(title.set_style(Color::White));
        frame.render_widget(&block, area);
        render_pane_content(frame, area, &block, pane, state)?;
    }

    if state.show_debug {
        render_debug_frame(frame, debug, state);
    }

    Ok(())
}

fn pane_title(pane: Pane, state: &TuiState) -> String {
    match pane {
        Pane::Profiles => format!("Profiles ({})", state.profiles.len()),
        Pane::Diff => {
            let (old, new) = state.selected_profiles();
            format!("Diff ({} -> {})", old.generation, new.generation)
        }
    }
}

/// Returns a bordered [`Block`] styled according to whether its pane is selected.
fn pane_block(selected: bool) -> Block<'static> {
    let (border_type, border_color) = if selected {
        (BorderType::Thick, Color::Gray)
    } else {
        (BorderType::Plain, Color::DarkGray)
    };
    Block::bordered()
        .merge_borders(MergeStrategy::Exact)
        .border_type(border_type)
        .border_style(border_color)
}

/// Renders pane content.
///
/// `area` is the outer pane area, including `block`'s borders,
/// so `block.inner(area)` is the area to render content into.
fn render_pane_content(
    frame: &mut Frame,
    area: Rect,
    block: &Block,
    pane: Pane,
    state: &mut TuiState,
) -> Result<()> {
    match pane {
        Pane::Profiles => {
            let pane = ProfilesPane::new(&state.profiles);
            frame.render_stateful_widget(pane, block.inner(area), &mut state.profiles_pane);
        }
        Pane::Diff => {
            let diff = state.diff_cache.get(&state.profiles_pane.selected());
            frame.render_stateful_widget(
                DiffPane::default(),
                block.inner(area),
                &mut state.diff_pane.with_diff(diff),
            );
        }
    }

    Ok(())
}

fn render_debug_frame(frame: &mut Frame, area: Rect, state: &mut TuiState) {
    let block = pane_block(true).title("Debug");
    frame.render_widget(&block, area);

    let mut debug_text = Text::default();
    debug_text.push_line(format!("Focus: {:?}", state.selected_pane));
    debug_text.push_line(format!("{:?}", state.profiles_pane));
    debug_text.push_line(format!("{:?}", state.diff_pane));
    let (old, new) = state.selected_profiles();
    debug_text.push_line(format!("Old ({}): {:?}", old.generation, old.path));
    debug_text.push_line(format!("New ({}): {:?}", new.generation, new.path));
    debug_text.push_line(format!("Diff cache size: {}", state.diff_cache.len(),));

    frame.render_widget(debug_text, block.inner(area));
}
