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
/// True when the exercise name embeds rep-phase semantics that belong on sets.
pub fn name_contains_phase_info(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.contains(ECCENTRIC) || lower.contains(CONCENTRIC) {
        return true;
    }
    name.split_whitespace().any(|word| {
        let token = word.trim_matches(|c: char| !c.is_alphanumeric());
        matches!(token, "ecc" | "conc")
    })
}

pub fn validate_exercise_name_phase(name: &str, allow_phase_in_name: bool) -> Result<()> {
    if allow_phase_in_name || !name_contains_phase_info(name) {
        return Ok(());
    }
    Err(RepslogError::Cli(
        "Exercise name contains rep phase information (eccentric/concentric). \
         Use one exercise per movement and tag sets with --phase full|eccentric|concentric instead. \
         Pass --allow-phase-in-name to override."
            .into(),
    ))
}

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
    fn detects_phase_words_in_exercise_names() {
        assert!(name_contains_phase_info("pistol squat (eccentric only)"));
        assert!(name_contains_phase_info("Concentric Press"));
        assert!(!name_contains_phase_info("pistol squat"));
    }

    #[test]
    fn validate_exercise_name_phase_rejects_without_override() {
        assert!(validate_exercise_name_phase("pistol squat (eccentric only)", false).is_err());
        assert!(validate_exercise_name_phase("pistol squat (eccentric only)", true).is_ok());
    }

    #[test]
    fn formats_non_full_phases() {
        assert_eq!(format_phase_label(FULL), "");
        assert_eq!(format_phase_label(ECCENTRIC), "eccentric");
        assert_eq!(format_reps_with_phase(3, ECCENTRIC), "3 reps (eccentric)");
    }
}
