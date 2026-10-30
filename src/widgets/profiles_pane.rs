use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::{List, ListState, StatefulWidget},
};

use crate::profile::Profile;

#[derive(Debug, Clone)]
pub struct ProfilesPane<'a> {
    pub profiles: &'a [Profile],
}

impl<'a> ProfilesPane<'a> {
    pub fn new(profiles: &'a [Profile]) -> Self {
        Self { profiles }
    }
}

impl<'a> StatefulWidget for ProfilesPane<'a> {
    type State = ProfilesPaneState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (old_idx, new_idx) = state.selected();
        let list = List::new(self.profiles.iter().enumerate().map(|(idx, it)| {
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

        list.render(area, buf, &mut state.list_state);
    }
}

#[derive(Debug, Clone)]
pub struct ProfilesPaneState {
    /// Total number of profiles
    profile_count: usize,
    /// List widget state
    list_state: ListState,
    /// Pinned profile index
    pinned_idx: Option<usize>,
}

impl ProfilesPaneState {
    pub fn new(profile_count: usize) -> Self {
        let last_profile_idx = profile_count.saturating_sub(1);

        Self {
            profile_count,
            list_state: ListState::default().with_selected(Some(last_profile_idx)),
            pinned_idx: None,
        }
    }

    /// Select the next profile.
    pub fn next(&mut self) {
        self.change_profile_selection(ListState::select_next)
    }

    /// Select the previous profile.
    pub fn prev(&mut self) {
        self.change_profile_selection(ListState::select_previous)
    }

    /// Select the first profile.
    pub fn first(&mut self) {
        self.change_profile_selection(ListState::select_first)
    }

    /// Select the last profile.
    pub fn last(&mut self) {
        self.change_profile_selection(ListState::select_last)
    }

    /// Applies `f` to the profile list and clamps the selection to the last index.
    fn change_profile_selection(&mut self, f: impl FnOnce(&mut ListState)) {
        let last_idx = self.profile_count.saturating_sub(1);
        f(&mut self.list_state);
        if let Some(idx) = self.list_state.selected() {
            // Clamp list selection to the last index
            self.list_state.select(Some(idx.min(last_idx)));
        }
    }

    /// Toggles the pinned profile index between the current selection and `None`.
    pub fn handle_pin_profile(&mut self) {
        self.pinned_idx = match self.pinned_idx {
            None => Some(self.list_state.selected().unwrap_or(0)),
            Some(_) => None,
        };
    }

    /// Returns selected old/new profile indexes.
    ///
    /// If there's only one profile, it is returned as both old and new.
    pub fn selected(&self) -> (usize, usize) {
        let new = self.list_state.selected().unwrap_or(0);
        let old = self.pinned_idx.unwrap_or_else(|| new.saturating_sub(1));
        (old, new)
    }
}
