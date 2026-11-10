//! A changelog release as a card that opens and shuts.
//!
//! The card's header is the caret, the version, the date and an optional
//! status tag at the right end, over a rule that runs the card's full width;
//! the notes sit under it. A click anywhere on the header opens or shuts the
//! notes through the caller's [`CollapseSet`], so a
//! [`CollapseAll`](crate::CollapseAll) over the same set works on every card.
//! Every site that shows a changelog draws its releases with this one card,
//! so they read the same. Styling lives in `assets/components.css` under
//! `.release-card`.

use dioxus::prelude::*;

use crate::{BannerStatus, CollapseBody, CollapseHeader, CollapseSet};

/// The pill at the right end of a release's header, such as "Upcoming" or
/// "Latest". `status` picks its color, as on a banner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseTag {
    /// The pill's color: amber for [`BannerStatus::Doing`], green for
    /// [`BannerStatus::Done`].
    pub status: BannerStatus,
    /// The pill's text.
    pub label: String,
}

/// One release: `version` with its `date` and `tag` in the header and
/// `entries` as the notes. `set` holds which releases are shut, keyed by
/// `version`.
///
/// `id` goes on the card so a URL fragment can land on it; the card's
/// `scroll-margin-top` reads `--release-card-scroll-margin`, which defaults
/// to the `--nav-height` a sticky nav sets plus 1rem. With `href` the version
/// is a link (usually `#` and the `id`), and following it leaves the card
/// open or shut as it was.
#[component]
pub fn ReleaseCard(
    set: CollapseSet,
    version: String,
    #[props(default)] date: Option<String>,
    #[props(default)] tag: Option<ReleaseTag>,
    entries: Vec<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] href: Option<String>,
) -> Element {
    rsx! {
        div { class: "release-card", id,
            CollapseHeader {
                set,
                section: version.clone(),
                label: version.clone(),
                class: "release-card-head collapse-header-quiet",
                h2 { class: "release-card-version",
                    if let Some(href) = href {
                        a {
                            class: "release-card-link",
                            href,
                            onclick: |event: MouseEvent| event.stop_propagation(),
                            "{version}"
                        }
                    } else {
                        "{version}"
                    }
                }
                if let Some(date) = date {
                    span { class: "release-card-date", "{date}" }
                }
                if let Some(tag) = tag {
                    span { class: "{tag.status.class()} release-card-tag", "{tag.label}" }
                }
            }
            CollapseBody { set, section: version,
                ul { class: "release-card-notes",
                    for entry in entries {
                        li { "{entry}" }
                    }
                }
            }
        }
    }
}
