use crate::error::RepslogError;
use crate::models::HeartRateZones;
use colored::*;
use comfy_table::Table;
use serde::Serialize;
use std::io::{self, Read};

/// Header underline only — no outer borders, column dividers, or row separators.
const HEADER_ONLY_PRESET: &str = "    ──              ";

pub const DATETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

/// Parse and validate a datetime string in `YYYY-MM-DD HH:MM:SS` format.
pub fn parse_datetime(s: &str) -> Result<String, RepslogError> {
    chrono::NaiveDateTime::parse_from_str(s, DATETIME_FMT)
        .map(|dt| dt.format(DATETIME_FMT).to_string())
        .map_err(|_| {
            RepslogError::Cli(
                "Invalid datetime format. Use YYYY-MM-DD HH:MM:SS (e.g. 2026-04-23 14:30:00)"
                    .to_string(),
            )
        })
}

/// Format a stored datetime for display, normalizing legacy date-only values.
pub fn format_datetime(s: &str) -> String {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, DATETIME_FMT) {
        return dt.format(DATETIME_FMT).to_string();
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return format!("{} 00:00:00", d.format("%Y-%m-%d"));
    }
    s.to_string()
}

pub fn format_datetime_opt(s: &Option<String>) -> Option<String> {
    s.as_ref().map(|v| format_datetime(v))
}

pub fn read_stdin() -> Option<String> {
    if !atty::is(atty::Stream::Stdin) {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() {
            let s = buffer.trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.load_preset(HEADER_ONLY_PRESET);
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    for column in table.column_iter_mut() {
        column.set_padding((0, 1));
    }
    println!("{}", table.trim_fmt());
}

pub fn format_duration(seconds: u32) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

pub fn format_pace(min_per_km: f64) -> String {
    let mins = min_per_km.floor() as u32;
    let secs = ((min_per_km - mins as f64) * 60.0).round() as u32;
    format!("{}'{:02}\"/km", mins, secs)
}

pub fn format_hr_zones_bar(zones: &HeartRateZones) -> String {
    let total_secs: u32 = zones.z1_seconds
        + zones.z2_seconds
        + zones.z3_seconds
        + zones.z4_seconds
        + zones.z5_seconds;
    if total_secs == 0 {
        return "No HR data".to_string();
    }

    let width: usize = 20;
    let z1_p = zones.z1_seconds as f64 / total_secs as f64;
    let z2_p = zones.z2_seconds as f64 / total_secs as f64;
    let z3_p = zones.z3_seconds as f64 / total_secs as f64;
    let z4_p = zones.z4_seconds as f64 / total_secs as f64;
    let z5_p = zones.z5_seconds as f64 / total_secs as f64;

    let z1_w = (z1_p * width as f64).round() as usize;
    let z2_w = (z2_p * width as f64).round() as usize;
    let z3_w = (z3_p * width as f64).round() as usize;
    let z4_w = (z4_p * width as f64).round() as usize;
    let z5_w = width.saturating_sub(z1_w + z2_w + z3_w + z4_w);

    let bar = format!(
        "{}{}{}{}{}",
        "█".repeat(z1_w).cyan(),
        "█".repeat(z2_w).green(),
        "█".repeat(z3_w).yellow(),
        "█".repeat(z4_w).magenta(),
        "█".repeat(z5_w).red()
    );

    format!(
        "{} (Z1:{:.0}% Z2:{:.0}% Z3:{:.0}% Z4:{:.0}% Z5:{:.0}%)",
        bar,
        z1_p * 100.0,
        z2_p * 100.0,
        z3_p * 100.0,
        z4_p * 100.0,
        z5_p * 100.0
    )
}

pub fn format_dry_run_id(id: i64, dry_run: bool) -> String {
    if dry_run {
        format!("DRY-RUN-{}", id)
    } else {
        id.to_string()
    }
}

pub fn parse_id(id_str: &str, dry_run: bool) -> Result<i64, crate::error::RepslogError> {
    if id_str.starts_with("DRY-RUN-") {
        if !dry_run {
            return Err(crate::error::RepslogError::Cli(format!(
                "ID '{}' is a dry-run ID and can only be used with the --dry-run flag",
                id_str
            )));
        }
        // For dry-run, we return a dummy ID as it won't be used for actual DB writes
        Ok(0)
    } else {
        id_str.parse::<i64>().map_err(|_| {
            crate::error::RepslogError::Cli(format!(
                "Invalid ID: '{}'. Must be an integer.",
                id_str
            ))
        })
    }
}

/// Print a serializable value as pretty JSON to stdout.
pub fn print_json<T: Serialize>(value: &T) -> crate::error::Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    println!("{}", s);
    Ok(())
}

/// Print a bare ID (for create/add results used in piping) or JSON {"id": "..."} when json=true.
/// Uses string for id to uniformly support DRY-RUN-N values.
pub fn print_id(id: &str, json: bool) {
    if json {
        println!(r#"{{"id": "{}"}}"#, id);
    } else {
        println!("{}", id);
    }
}

/// Normalize a custom exercise name for storage: trim and collapse whitespace.
/// Rejects names containing uppercase letters (Title Case, CamelCase, etc.).
pub fn normalize_exercise_name(name: &str) -> Result<String, RepslogError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(RepslogError::Cli(
            "Exercise name cannot be empty".to_string(),
        ));
    }
    if trimmed.chars().any(|c| c.is_uppercase()) {
        return Err(RepslogError::Cli(format!(
            "Exercise names must be lowercase. Use: {}",
            trimmed.to_lowercase()
        )));
    }
    Ok(trimmed.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// If `name` contains likely plural words, return a singular suggestion.
pub fn suggest_singular_exercise_name(name: &str) -> Option<String> {
    let mut changed = false;
    let words: Vec<String> = name
        .split_whitespace()
        .map(|word| {
            if is_likely_plural_word(word) {
                changed = true;
                singularize_word(word)
            } else {
                word.to_string()
            }
        })
        .collect();
    changed.then(|| words.join(" "))
}

fn is_likely_plural_word(word: &str) -> bool {
    let alnum: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
    alnum.len() >= 3 && alnum.ends_with('s') && !alnum.ends_with("ss")
}

fn singularize_word(word: &str) -> String {
    word.strip_suffix('s')
        .map(str::to_string)
        .unwrap_or_else(|| word.to_string())
}

/// Alphanumeric lowercase key for near-duplicate detection (strips trailing plural "s").
pub fn exercise_similarity_key(name: &str) -> String {
    let raw: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    raw.strip_suffix('s')
        .filter(|s| s.len() >= 3)
        .unwrap_or(&raw)
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExerciseNameConflictKind {
    Duplicate,
    Similar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseNameConflict {
    pub existing_id: i64,
    pub existing_name: String,
    pub kind: ExerciseNameConflictKind,
}

/// Find catalog entries that duplicate or closely resemble `new_name`.
pub fn find_exercise_name_conflicts(
    new_name: &str,
    existing: &[(i64, String)],
) -> Vec<ExerciseNameConflict> {
    let new_key = exercise_similarity_key(new_name);
    if new_key.is_empty() {
        return Vec::new();
    }

    let mut conflicts = Vec::new();
    for &(id, ref existing_name) in existing {
        let existing_key = exercise_similarity_key(existing_name);
        if existing_key.is_empty() {
            continue;
        }
        if new_key == existing_key {
            conflicts.push(ExerciseNameConflict {
                existing_id: id,
                existing_name: existing_name.clone(),
                kind: ExerciseNameConflictKind::Duplicate,
            });
            continue;
        }
        if exercise_names_similar(new_name, existing_name, &new_key, &existing_key) {
            conflicts.push(ExerciseNameConflict {
                existing_id: id,
                existing_name: existing_name.clone(),
                kind: ExerciseNameConflictKind::Similar,
            });
        }
    }
    conflicts
}

fn exercise_names_similar(a_name: &str, b_name: &str, a_key: &str, b_key: &str) -> bool {
    if a_key == b_key {
        return false;
    }
    let (shorter, longer) = if a_key.len() <= b_key.len() {
        (a_key, b_key)
    } else {
        (b_key, a_key)
    };
    if shorter.len() >= 5 && longer.contains(shorter) {
        return true;
    }
    if a_key.len() >= 5 && b_key.len() >= 5 && levenshtein_distance(a_key, b_key) <= 1 {
        return true;
    }
    exercise_name_tokens_overlap(a_name, b_name)
}

fn exercise_name_tokens(name: &str) -> Vec<String> {
    name.to_lowercase()
        .split_whitespace()
        .map(|word| {
            let alnum: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            alnum
                .strip_suffix('s')
                .filter(|s| s.len() >= 3)
                .unwrap_or(&alnum)
                .to_string()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn exercise_name_tokens_overlap(a_name: &str, b_name: &str) -> bool {
    use std::collections::HashSet;

    let a_tokens: HashSet<_> = exercise_name_tokens(a_name).into_iter().collect();
    let b_tokens: HashSet<_> = exercise_name_tokens(b_name).into_iter().collect();
    if a_tokens.is_empty() || b_tokens.is_empty() {
        return false;
    }
    let shared = a_tokens.intersection(&b_tokens).count();
    let min_len = a_tokens.len().min(b_tokens.len());
    shared >= 2 && shared * 100 >= min_len * 80
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, a_ch) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b_chars.iter().enumerate() {
            let cost = usize::from(a_ch != b_ch);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_smoke() {
        // Just verify it doesn't panic
        let headers = vec!["ID", "Name"];
        let rows = vec![vec!["1".to_string(), "Test".to_string()]];
        print_table(headers, rows);
    }

    #[test]
    fn test_parse_datetime_valid() {
        assert_eq!(
            parse_datetime("2026-04-23 14:30:00").unwrap(),
            "2026-04-23 14:30:00"
        );
    }

    #[test]
    fn test_parse_datetime_rejects_date_only() {
        assert!(parse_datetime("2026-04-23").is_err());
    }

    #[test]
    fn test_format_datetime_normalizes_date_only() {
        assert_eq!(format_datetime("2026-04-23"), "2026-04-23 00:00:00");
        assert_eq!(
            format_datetime("2026-04-23 14:30:00"),
            "2026-04-23 14:30:00"
        );
    }

    #[test]
    fn test_normalize_exercise_name_lowercase() {
        assert_eq!(
            normalize_exercise_name("bulgarian split squat").unwrap(),
            "bulgarian split squat"
        );
        assert_eq!(normalize_exercise_name("  pull   up  ").unwrap(), "pull up");
    }

    #[test]
    fn test_suggest_singular_exercise_name() {
        assert_eq!(
            suggest_singular_exercise_name("pull ups"),
            Some("pull up".to_string())
        );
        assert_eq!(
            suggest_singular_exercise_name("bulgarian split squat"),
            None
        );
        assert_eq!(suggest_singular_exercise_name("bench press"), None);
    }

    #[test]
    fn test_normalize_exercise_name_rejects_uppercase() {
        assert!(normalize_exercise_name("Pull Ups").is_err());
        assert!(normalize_exercise_name("pullUps").is_err());
        assert!(normalize_exercise_name("").is_err());
    }

    #[test]
    fn test_exercise_similarity_key_plural_and_spacing() {
        assert_eq!(exercise_similarity_key("pull ups"), "pullup");
        assert_eq!(exercise_similarity_key("Pullups"), "pullup");
        assert_eq!(exercise_similarity_key("pull up"), "pullup");
    }

    #[test]
    fn test_find_exercise_name_conflicts_duplicate() {
        let existing = vec![(1, "Pullups".to_string())];
        let conflicts = find_exercise_name_conflicts("pull ups", &existing);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, ExerciseNameConflictKind::Duplicate);
    }

    #[test]
    fn test_find_exercise_name_conflicts_similar() {
        let existing = vec![(2, "Nordic Hamstring Curl".to_string())];
        let conflicts = find_exercise_name_conflicts("nordic curl", &existing);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, ExerciseNameConflictKind::Similar);
    }
}
