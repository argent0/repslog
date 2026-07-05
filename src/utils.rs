use crate::error::RepslogError;
use crate::models::HeartRateZones;
use colored::*;
use comfy_table::Table;
use serde::Serialize;
use std::io::{self, Read};

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
    let mut table = Table::new();
    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }
    println!("{}", table);
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
}
