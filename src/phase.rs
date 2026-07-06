use crate::error::{RepslogError, Result};

pub const FULL: &str = "full";
pub const ECCENTRIC: &str = "eccentric";
pub const CONCENTRIC: &str = "concentric";

pub fn normalize_phase(value: &str) -> Result<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "full" => Ok(FULL),
        "eccentric" | "ecc" => Ok(ECCENTRIC),
        "concentric" | "conc" => Ok(CONCENTRIC),
        other => Err(RepslogError::Cli(format!(
            "Invalid phase '{other}'. Expected one of: full, eccentric, concentric."
        ))),
    }
}

/// Short label for tables; returns empty string for full reps.
pub fn format_phase_label(phase: &str) -> String {
    match phase {
        ECCENTRIC => "eccentric".to_string(),
        CONCENTRIC => "concentric".to_string(),
        _ => String::new(),
    }
}

/// Detail suffix for rep counts, e.g. "8 reps (eccentric)".
pub fn format_reps_with_phase(reps: i32, phase: &str) -> String {
    match phase {
        ECCENTRIC => format!("{reps} reps (eccentric)"),
        CONCENTRIC => format!("{reps} reps (concentric)"),
        _ => format!("{reps} reps"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_aliases() {
        assert_eq!(normalize_phase("ecc").unwrap(), ECCENTRIC);
        assert_eq!(normalize_phase("conc").unwrap(), CONCENTRIC);
        assert_eq!(normalize_phase("FULL").unwrap(), FULL);
    }

    #[test]
    fn rejects_unknown_phase() {
        assert!(normalize_phase("isometric").is_err());
    }

    #[test]
    fn formats_non_full_phases() {
        assert_eq!(format_phase_label(FULL), "");
        assert_eq!(format_phase_label(ECCENTRIC), "eccentric");
        assert_eq!(format_reps_with_phase(3, ECCENTRIC), "3 reps (eccentric)");
    }
}
