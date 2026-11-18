//! A keyed toast's live text: a toast pushed under a key that is already showing changes that
//! toast's text in place rather than adding one. The text keeps its previous value, so the part
//! that changed can crossfade from the old to the new while the rest holds still.

use std::mem;

/// What starts the title a keyed toast is pushed to the toast library under, ahead of its key: a
/// control character no real title begins with.
pub const LIVE_MARK: &str = "\u{1f}live:";

/// The title a toast under `key` is pushed to the toast library with.
#[must_use]
pub fn live_title(key: &str) -> String {
    format!("{LIVE_MARK}{key}")
}

/// The key a library title carries, when it is a keyed toast's.
#[must_use]
pub fn live_key(title: &str) -> Option<&str> {
    title.strip_prefix(LIVE_MARK)
}

/// A keyed toast's text, the text it had before its last update, and how many updates it has had.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiveText {
    text: String,
    prev: Option<String>,
    updates: u32,
}

/// A keyed toast's text split for its crossfade: the start the old and new text share, then the
/// old text's rest fading out over the new text's rest fading in. Both rests are empty before the
/// first update and when an update changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Change<'a> {
    /// The start that holds still.
    pub kept: &'a str,
    /// The old text after the start, fading out.
    pub old: &'a str,
    /// The new text after the start, fading in.
    pub new: &'a str,
}

impl LiveText {
    /// A keyed toast's first text.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self {
            text,
            prev: None,
            updates: 0,
        }
    }

    /// Takes `next` as the text, keeping the one it replaces for the crossfade.
    pub fn update(&mut self, next: String) {
        self.prev = Some(mem::replace(&mut self.text, next));
        self.updates = self.updates.saturating_add(1);
    }

    /// The text shown.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// How many updates the toast has had: 0 when it first lands.
    #[must_use]
    pub fn updates(&self) -> u32 {
        self.updates
    }

    /// The text split for its crossfade (see [`Change`]).
    #[must_use]
    pub fn change(&self) -> Change<'_> {
        let Some(prev) = self.prev.as_deref() else {
            return Change {
                kept: &self.text,
                old: "",
                new: "",
            };
        };
        // The byte length of the longest shared start, ending on a character boundary in both.
        let shared = self
            .text
            .char_indices()
            .zip(prev.chars())
            .find(|((_, a), b)| a != b)
            .map_or_else(|| self.text.len().min(prev.len()), |((at, _), _)| at);
        Change {
            kept: self.text.get(..shared).unwrap_or_default(),
            old: prev.get(shared..).unwrap_or_default(),
            new: self.text.get(shared..).unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyed_title_carries_its_key_and_a_plain_one_none() {
        assert_eq!(
            live_key(&live_title("tally:Pushups")),
            Some("tally:Pushups")
        );
        assert_eq!(live_key("Saved"), None);
        assert!(live_title("k").starts_with(LIVE_MARK));
    }

    #[test]
    fn a_toast_lands_whole_with_nothing_to_crossfade() {
        let live = LiveText::new("Pull-ups +1".to_owned());
        assert_eq!(live.updates(), 0);
        assert_eq!(
            live.change(),
            Change {
                kept: "Pull-ups +1",
                old: "",
                new: ""
            }
        );
    }

    #[test]
    fn an_update_crossfades_only_the_part_that_changed() {
        let mut live = LiveText::new("Pull-ups +1".to_owned());
        live.update("Pull-ups +2".to_owned());
        assert_eq!(live.updates(), 1);
        assert_eq!(live.text(), "Pull-ups +2");
        assert_eq!(
            live.change(),
            Change {
                kept: "Pull-ups +",
                old: "1",
                new: "2"
            }
        );
        for n in 3..=10 {
            live.update(format!("Pull-ups +{n}"));
        }
        assert_eq!(live.updates(), 9);
        assert_eq!(
            live.change(),
            Change {
                kept: "Pull-ups +",
                old: "9",
                new: "10"
            }
        );
    }

    #[test]
    fn the_split_lands_on_a_character_boundary() {
        let mut live = LiveText::new("Café +1".to_owned());
        live.update("Cafè +1".to_owned());
        assert_eq!(
            live.change(),
            Change {
                kept: "Caf",
                old: "é +1",
                new: "è +1"
            }
        );
    }
}
