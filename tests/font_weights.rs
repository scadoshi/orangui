//! Every weight a sheet names is a face a host ships: `JetBrains Mono` comes in 400, 500 and 700,
//! and a browser draws 600 with the 700 face, so a rule says 700 when it means bold.

/// Every stylesheet the kit ships, by file name.
const SHEETS: [(&str, &str); 8] = [
    ("themes.css", include_str!("../assets/themes.css")),
    ("components.css", include_str!("../assets/components.css")),
    ("app.css", include_str!("../assets/app.css")),
    ("site.css", include_str!("../assets/site.css")),
    ("toast.css", include_str!("../assets/toast.css")),
    ("dialog.css", include_str!("../assets/dialog.css")),
    ("nav_fold.css", include_str!("../assets/nav_fold.css")),
    ("screen.css", include_str!("../assets/screen.css")),
];

#[test]
fn no_sheet_asks_for_a_weight_that_draws_as_another() {
    for (file, sheet) in SHEETS {
        for (number, line) in sheet.lines().enumerate() {
            let squeezed: String = line.split_whitespace().collect();
            assert!(
                !squeezed.contains("font-weight:600"),
                "{file}:{}: 600 draws with the 700 face; say 700",
                number.saturating_add(1)
            );
        }
    }
}
