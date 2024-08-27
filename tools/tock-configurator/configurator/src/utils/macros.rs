// Copyright OxidOS Automotive 2024.

/// The [`submenu!`] macro adds the submenu symbol `⎯⎯>` at the end of the given string.
#[macro_export]
macro_rules! submenu {
    ($name:literal) => {
        std::format!("{} -->", $name)
    };
}

/// The [`capsule!`] macro adds the capsule config symbol `[ ]`/`[*]` at the beggining of the given string,
/// as well as the submenu symbel `⎯⎯>` at the end of the given string.
#[macro_export]
macro_rules! capsule {
    ($name:literal, $checked:expr) => {
        if $checked {
            std::format!("[*] {} -->", $name)
        } else {
            std::format!("[ ] {} -->", $name)
        }
    };
}
