use std::env;

pub(crate) fn is_debug_enabled() -> bool {
    if let Ok(val) = env::var("GOMOKU_DEBUG") {
        val == "1" || val.eq_ignore_ascii_case("true")
    } else {
        false
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug_enabled() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}
