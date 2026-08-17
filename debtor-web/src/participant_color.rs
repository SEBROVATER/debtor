//! Suggested participant colors.

use std::sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
};

const COLORS: [&str; 12] = [
    "#16697A", "#C44536", "#6A4C93", "#2A9D8F", "#D97706", "#3A5A40", "#B23A48", "#2563EB",
    "#7C3AED", "#0F766E", "#BE123C", "#4D7C0F",
];

/// Returns a varied valid color for a fresh participant form.
pub(crate) fn suggested_participant_color() -> &'static str {
    static NEXT: OnceLock<AtomicUsize> = OnceLock::new();
    let next = NEXT.get_or_init(|| AtomicUsize::new(0));
    COLORS[next.fetch_add(1, Ordering::Relaxed) % COLORS.len()]
}

#[cfg(test)]
mod tests {
    use debtor_domain::model::Color;

    use super::{COLORS, suggested_participant_color};

    #[test]
    fn suggestions_are_valid_and_do_not_repeat_consecutively() {
        let first = suggested_participant_color();
        let second = suggested_participant_color();
        assert!(COLORS.contains(&first));
        assert!(COLORS.contains(&second));
        assert!(Color::new(first).is_ok());
        assert!(Color::new(second).is_ok());
        assert_ne!(first, second);
    }
}
