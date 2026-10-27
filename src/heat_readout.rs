//! A heatmap's readout: a row of tags under the grid saying what the year
//! adds up to, and what one cell holds once it is picked.
//!
//! Two ways to name a cell, and the stylesheet picks between them so a host
//! renders both and checks no platform. A hover device with room names the
//! cell under the pointer in a floating [`crate::ChartTip`] given the
//! `heat-float` class, and the readout keeps its resting tags. A touch or
//! narrow screen hides that tip; a tapped cell's tags take the readout's
//! place for [`HEAT_PICK_MS`] and then the resting tags ease back, so the
//! totals are never more than a few seconds away.

use std::fmt;

use dioxus::prelude::*;

use crate::entrance::sleep_ms;

/// How long a picked cell's tags hold before the resting tags come back.
pub const HEAT_PICK_MS: u32 = 4000;

/// The cell a heatmap's readout names, held for [`HEAT_PICK_MS`] after each
/// pick. Copy it into every handler that picks; [`HeatReadout`] reads it.
#[derive(Clone, Copy, PartialEq)]
pub struct HeatPick {
    tags: Signal<Option<Vec<String>>>,
    turn: Signal<u32>,
}

impl fmt::Debug for HeatPick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeatPick").finish_non_exhaustive()
    }
}

impl HeatPick {
    /// Shows `tags` in the readout, replacing any earlier pick, for
    /// [`HEAT_PICK_MS`] from now.
    pub fn show(mut self, tags: Vec<String>) {
        let turn = self.turn.peek().wrapping_add(1);
        self.turn.set(turn);
        self.tags.set(Some(tags));
        spawn(async move {
            sleep_ms(HEAT_PICK_MS).await;
            // A later pick owns the readout now.
            if *self.turn.peek() == turn {
                self.tags.set(None);
            }
        });
    }

    /// Puts the resting tags back at once.
    pub fn clear(mut self) {
        let turn = self.turn.peek().wrapping_add(1);
        self.turn.set(turn);
        self.tags.set(None);
    }

    /// The picked tags, if a pick is holding. Reading it subscribes the
    /// caller.
    #[must_use]
    pub fn tags(&self) -> Option<Vec<String>> {
        (self.tags)()
    }

    /// Which pick is showing, so a host can key on it.
    #[must_use]
    pub fn turn(&self) -> u32 {
        (self.turn)()
    }
}

/// A pick that starts empty, for one heatmap.
#[must_use]
pub fn use_heat_pick() -> HeatPick {
    HeatPick {
        tags: use_signal(|| None),
        turn: use_signal(|| 0),
    }
}

/// The row of tags under a heatmap: `rest`, the year's totals and its span,
/// or on a touch or narrow screen the tags of the cell `pick` holds, each
/// row fading up as it arrives. All in one muted tone, so the grid stays the
/// colorful part.
#[component]
pub fn HeatReadout(rest: Vec<String>, pick: HeatPick) -> Element {
    let picked = pick.tags();
    let turn = pick.turn();
    let class = if picked.is_some() {
        "heat-readout picking"
    } else {
        "heat-readout"
    };
    rsx! {
        div { class,
            div { class: "heat-readout-row heat-readout-rest chip-in",
                for (i, tag) in rest.iter().enumerate() {
                    span { key: "{i}", class: "tag heat-readout-tag", "{tag}" }
                }
            }
            if let Some(tags) = picked {
                // Keyed on the pick, so each one fades up afresh.
                for turn in [turn] {
                    div { key: "{turn}", class: "heat-readout-row heat-readout-pick chip-in",
                        for (i, tag) in tags.iter().enumerate() {
                            span { key: "{i}", class: "tag heat-readout-tag", "{tag}" }
                        }
                    }
                }
            }
        }
    }
}
