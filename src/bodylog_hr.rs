//! HR zone bounds from bodylog profile (DOB + sleep resting HR).
//!
//! Snapshots DOB and resting HR into repslog at import so bodylog is not required later.
//! Intermediate values (age, HRmax, zone bound array) are not stored — only the inputs
//! and the resulting `heart_rate_zones` time-in-zone on the set.

use crate::error::{RepslogError, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::process::Command;

/// Default lookback for sleep HR samples when estimating RHR.
pub const SLEEP_HR_LOOKBACK_DAYS: u32 = 14;

/// Profile inputs resolved from bodylog (to be stored on the set).
#[derive(Debug, Clone, PartialEq)]
pub struct HrZoneProfile {
    /// YYYY-MM-DD from bodylog user profile.
    pub date_of_birth: String,
    /// Median sleep HR over lookback, if available.
    pub resting_hr_bpm: Option<f64>,
    /// Upper bpm bounds for zones 1–5 (ephemeral; used only to compute time-in-zone).
    pub bounds: [f64; 5],
    /// Human-readable method for stderr diagnostics.
    pub method: String,
}

/// Median of a non-empty slice of f64 values. Returns None if empty.
pub fn median_f64(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        Some(v[n / 2])
    } else {
        Some((v[n / 2 - 1] + v[n / 2]) / 2.0)
    }
}

/// Whole years of age on `on_date` given DOB `YYYY-MM-DD`.
pub fn age_years(dob: &str, on_date: NaiveDate) -> Option<u32> {
    let dob = NaiveDate::parse_from_str(dob.trim(), "%Y-%m-%d").ok()?;
    if on_date < dob {
        return None;
    }
    let mut age = (on_date.year() - dob.year()) as u32;
    let birthday_passed = (on_date.month(), on_date.day()) >= (dob.month(), dob.day());
    if !birthday_passed {
        age = age.saturating_sub(1);
    }
    Some(age)
}

/// Tanaka HRmax estimate: 208 − 0.7 × age.
pub fn hr_max_tanaka(age: u32) -> f64 {
    208.0 - 0.7 * age as f64
}

const ZONE_FRACS: [f64; 5] = [0.60, 0.70, 0.80, 0.90, 1.00];

/// Karvonen upper bounds: RHR + p × (HRmax − RHR).
pub fn zone_bounds_karvonen(hr_rest: f64, hr_max: f64) -> Option<[f64; 5]> {
    if !hr_rest.is_finite() || !hr_max.is_finite() || hr_max <= hr_rest {
        return None;
    }
    let hrr = hr_max - hr_rest;
    let mut out = [0.0; 5];
    for (i, p) in ZONE_FRACS.iter().enumerate() {
        out[i] = (hr_rest + p * hrr).round();
    }
    ensure_nondecreasing(&mut out);
    Some(out)
}

/// Percent-of-HRmax upper bounds.
pub fn zone_bounds_pct_max(hr_max: f64) -> Option<[f64; 5]> {
    if !hr_max.is_finite() || hr_max <= 0.0 {
        return None;
    }
    let mut out = [0.0; 5];
    for (i, p) in ZONE_FRACS.iter().enumerate() {
        out[i] = (p * hr_max).round();
    }
    ensure_nondecreasing(&mut out);
    Some(out)
}

fn ensure_nondecreasing(bounds: &mut [f64; 5]) {
    for i in 1..5 {
        if bounds[i] < bounds[i - 1] {
            bounds[i] = bounds[i - 1];
        }
    }
}

/// Build auto bounds from DOB + optional sleep HR samples (median RHR).
///
/// - DOB + usable median RHR → Karvonen
/// - DOB only → %HRmax
/// - Invalid age/RHR → None
pub fn resolve_auto_bounds(
    date_of_birth: &str,
    on_date: NaiveDate,
    sleep_hrs: &[f64],
) -> Option<HrZoneProfile> {
    let age = age_years(date_of_birth, on_date)?;
    if !(10..=100).contains(&age) {
        return None;
    }
    let hr_max = hr_max_tanaka(age);
    let rhr = median_f64(sleep_hrs).and_then(|m| {
        if (30.0..=100.0).contains(&m) {
            Some(m)
        } else {
            None
        }
    });

    if let Some(rest) = rhr {
        let bounds = zone_bounds_karvonen(rest, hr_max)?;
        Some(HrZoneProfile {
            date_of_birth: date_of_birth.trim().to_string(),
            resting_hr_bpm: Some(rest),
            bounds,
            method: format!(
                "bodylog Karvonen (age {}, RHR median {:.0}, HRmax {:.0})",
                age, rest, hr_max
            ),
        })
    } else {
        let bounds = zone_bounds_pct_max(hr_max)?;
        Some(HrZoneProfile {
            date_of_birth: date_of_birth.trim().to_string(),
            resting_hr_bpm: None,
            bounds,
            method: format!("bodylog %HRmax (age {}, HRmax {:.0})", age, hr_max),
        })
    }
}

// --- bodylog CLI ---

#[derive(Debug, Deserialize)]
struct BodylogConfigShow {
    date_of_birth: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BodylogSleepEntry {
    heart_rate_bpm: Option<f64>,
}

/// Fetch DOB + sleep HR samples from the `bodylog` CLI.
/// Hard-fails with a clear CLI error if bodylog is missing or unusable.
pub fn fetch_bodylog_hr_inputs() -> Result<(String, Vec<f64>)> {
    let config_out = run_bodylog(&["config", "show", "--json"])?;
    let config: BodylogConfigShow = serde_json::from_str(&config_out).map_err(|e| {
        RepslogError::Cli(format!(
            "Failed to parse `bodylog config show --json`: {}. \
             Fix bodylog profile or pass --no-bodylog / --hr-zone-bounds.",
            e
        ))
    })?;
    let dob = config
        .date_of_birth
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            RepslogError::Cli(
                "bodylog has no date_of_birth set (needed for HR zones). \
                 Run: bodylog config set --date-of-birth YYYY-MM-DD \
                 Or pass --hr-zone-bounds / --no-bodylog."
                    .into(),
            )
        })?
        .to_string();

    // Validate DOB format early
    if NaiveDate::parse_from_str(&dob, "%Y-%m-%d").is_err() {
        return Err(RepslogError::Cli(format!(
            "bodylog date_of_birth '{}' is not YYYY-MM-DD",
            dob
        )));
    }

    let sleep_out = run_bodylog(&[
        "sleep",
        "list",
        "--days",
        &SLEEP_HR_LOOKBACK_DAYS.to_string(),
        "--json",
    ])?;
    let entries: Vec<BodylogSleepEntry> = serde_json::from_str(&sleep_out).map_err(|e| {
        RepslogError::Cli(format!(
            "Failed to parse `bodylog sleep list --json`: {}",
            e
        ))
    })?;
    let sleep_hrs: Vec<f64> = entries
        .iter()
        .filter_map(|e| e.heart_rate_bpm)
        .filter(|h| h.is_finite() && *h > 0.0)
        .collect();

    Ok((dob, sleep_hrs))
}

/// Resolve profile + bounds from bodylog for an activity date (`YYYY-MM-DD` or datetime prefix).
pub fn resolve_from_bodylog(activity_started_at: &str) -> Result<HrZoneProfile> {
    let on_date = parse_activity_date(activity_started_at).ok_or_else(|| {
        RepslogError::Cli(format!(
            "Cannot parse activity date from '{}' for HR zone age",
            activity_started_at
        ))
    })?;
    let (dob, sleep_hrs) = fetch_bodylog_hr_inputs()?;
    resolve_auto_bounds(&dob, on_date, &sleep_hrs).ok_or_else(|| {
        RepslogError::Cli(
            "Could not derive HR zone bounds from bodylog (check age range and sleep HR). \
             Pass --hr-zone-bounds or --no-bodylog."
                .into(),
        )
    })
}

fn parse_activity_date(started_at: &str) -> Option<NaiveDate> {
    let s = started_at.trim();
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    // "YYYY-MM-DD HH:MM:SS"
    if s.len() >= 10 {
        return NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok();
    }
    None
}

fn run_bodylog(args: &[&str]) -> Result<String> {
    let output = Command::new("bodylog").args(args).output().map_err(|e| {
        RepslogError::Cli(format!(
            "Failed to run `bodylog` ({}). Install bodylog, or pass --no-bodylog / --hr-zone-bounds.",
            e
        ))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepslogError::Cli(format!(
            "`bodylog {}` failed (status {}): {}. \
             Fix bodylog, or pass --no-bodylog / --hr-zone-bounds.",
            args.join(" "),
            output.status,
            stderr.trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| RepslogError::Cli(format!("bodylog output is not valid UTF-8: {}", e)))
}

// Need Datelike for year/month/day
use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_empty() {
        assert_eq!(median_f64(&[]), None);
    }

    #[test]
    fn median_odd() {
        assert_eq!(median_f64(&[3.0, 1.0, 2.0]), Some(2.0));
    }

    #[test]
    fn median_even() {
        assert_eq!(median_f64(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
    }

    #[test]
    fn age_before_birthday() {
        let dob = "1983-07-21";
        let on = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        assert_eq!(age_years(dob, on), Some(42)); // birthday later in July
    }

    #[test]
    fn age_on_birthday() {
        let dob = "1983-07-21";
        let on = NaiveDate::from_ymd_opt(2026, 7, 21).unwrap();
        assert_eq!(age_years(dob, on), Some(43));
    }

    #[test]
    fn karvonen_monotonic() {
        let b = zone_bounds_karvonen(52.0, 179.0).unwrap();
        for i in 1..5 {
            assert!(b[i] >= b[i - 1], "{:?}", b);
        }
        assert!((b[4] - 179.0).abs() < 1.0);
    }

    #[test]
    fn pct_max_only() {
        let on = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        let p = resolve_auto_bounds("1983-07-21", on, &[]).unwrap();
        assert!(p.resting_hr_bpm.is_none());
        assert!(p.method.contains("%HRmax"));
        assert_eq!(p.date_of_birth, "1983-07-21");
    }

    #[test]
    fn karvonen_with_sleep() {
        let on = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        let p = resolve_auto_bounds("1983-07-21", on, &[52.0, 50.0, 54.0]).unwrap();
        assert_eq!(p.resting_hr_bpm, Some(52.0));
        assert!(p.method.contains("Karvonen"));
    }

    #[test]
    fn invalid_rhr_falls_back_to_pct() {
        let on = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        let p = resolve_auto_bounds("1983-07-21", on, &[10.0]).unwrap(); // below 30
        assert!(p.resting_hr_bpm.is_none());
    }

    #[test]
    fn bad_age_none() {
        let on = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        assert!(resolve_auto_bounds("2019-01-01", on, &[52.0]).is_none()); // age ~7
    }
}
