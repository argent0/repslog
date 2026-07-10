//! Absolute range validation (hard-fail) for inserted training metrics.
//!
//! No variation / delta layer — only physical range and cross-field consistency.
//! Limits come from `AppConfig.sanity` (defaults or `~/.config/repslog/config.toml`).

use crate::app_config::{AbsoluteLimits, SanityLimits};
use crate::error::{RepslogError, Result};
use crate::models::{HeartRateZones, Lap, Trackpoint};

/// Metrics proposed on set create/update (only fields that were supplied).
#[derive(Debug, Clone, Default)]
pub struct ProposedSetMetrics {
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub external_load_kg: Option<f64>,
    pub distance_km: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub rir: Option<f64>,
    pub effective_reps: Option<i32>,
    pub rest_seconds: Option<i32>,
    pub avg_heart_rate_bpm: Option<f64>,
    pub max_heart_rate_bpm: Option<f64>,
    pub avg_pace_min_per_km: Option<f64>,
    pub calories_burned: Option<i32>,
    pub avg_cadence_spm: Option<f64>,
    pub total_ascent_m: Option<f64>,
    pub total_descent_m: Option<f64>,
    pub heart_rate_zones: Option<HeartRateZones>,
    pub laps: Option<Vec<Lap>>,
}

/// Workout-level metrics on create/update.
#[derive(Debug, Clone, Default)]
pub struct ProposedWorkoutMetrics {
    pub duration_minutes: Option<i32>,
    pub overall_feeling: Option<i32>,
}

/// Hard-fail absolute impossibilities using configured limits.
pub fn validate_set_metrics(new: &ProposedSetMetrics, limits: &SanityLimits) -> Result<()> {
    let mut errors = Vec::new();

    check_i32_range("reps", new.reps, &limits.reps, &mut errors);
    check_f64_range("weight_kg", new.weight_kg, &limits.weight_kg, &mut errors);
    check_f64_range(
        "external_load_kg",
        new.external_load_kg,
        &limits.external_load_kg,
        &mut errors,
    );
    check_i32_range(
        "duration_seconds",
        new.duration_seconds,
        &limits.duration_seconds,
        &mut errors,
    );
    check_f64_range(
        "distance_km",
        new.distance_km,
        &limits.distance_km,
        &mut errors,
    );
    check_f64_range("rpe", new.rpe, &limits.rpe, &mut errors);
    check_f64_range("rir", new.rir, &limits.rir, &mut errors);
    check_i32_range(
        "effective_reps",
        new.effective_reps,
        &limits.effective_reps,
        &mut errors,
    );
    check_i32_range(
        "rest_seconds",
        new.rest_seconds,
        &limits.rest_seconds,
        &mut errors,
    );
    check_f64_range(
        "avg_heart_rate_bpm",
        new.avg_heart_rate_bpm,
        &limits.heart_rate_bpm,
        &mut errors,
    );
    check_f64_range(
        "max_heart_rate_bpm",
        new.max_heart_rate_bpm,
        &limits.heart_rate_bpm,
        &mut errors,
    );
    check_f64_range(
        "avg_pace_min_per_km",
        new.avg_pace_min_per_km,
        &limits.pace_min_per_km,
        &mut errors,
    );
    check_i32_range(
        "calories_burned",
        new.calories_burned,
        &limits.calories_burned,
        &mut errors,
    );
    check_f64_range(
        "avg_cadence_spm",
        new.avg_cadence_spm,
        &limits.cadence_spm,
        &mut errors,
    );
    check_f64_range(
        "total_ascent_m",
        new.total_ascent_m,
        &limits.elevation_m,
        &mut errors,
    );
    check_f64_range(
        "total_descent_m",
        new.total_descent_m,
        &limits.elevation_m,
        &mut errors,
    );

    if let (Some(avg), Some(max)) = (new.avg_heart_rate_bpm, new.max_heart_rate_bpm) {
        if avg.is_finite() && max.is_finite() && avg > max {
            errors.push(format!(
                "avg_heart_rate_bpm ({}) must be <= max_heart_rate_bpm ({})",
                format_num(avg),
                format_num(max)
            ));
        }
    }

    if let (Some(reps), Some(eff)) = (new.reps, new.effective_reps) {
        if eff > reps {
            errors.push(format!(
                "effective_reps ({}) must be <= reps ({})",
                eff, reps
            ));
        }
    }

    if let Some(ref zones) = new.heart_rate_zones {
        check_zone_seconds("z1_seconds", zones.z1_seconds, limits, &mut errors);
        check_zone_seconds("z2_seconds", zones.z2_seconds, limits, &mut errors);
        check_zone_seconds("z3_seconds", zones.z3_seconds, limits, &mut errors);
        check_zone_seconds("z4_seconds", zones.z4_seconds, limits, &mut errors);
        check_zone_seconds("z5_seconds", zones.z5_seconds, limits, &mut errors);
        if let Some(dur) = new.duration_seconds {
            let sum = zones.z1_seconds as u64
                + zones.z2_seconds as u64
                + zones.z3_seconds as u64
                + zones.z4_seconds as u64
                + zones.z5_seconds as u64;
            let cap = ((dur as f64) * 1.1).ceil() as u64;
            if sum > cap {
                errors.push(format!(
                    "heart_rate_zones sum ({} s) exceeds duration_seconds * 1.1 ({} s)",
                    sum, cap
                ));
            }
        }
    }

    if let Some(ref laps) = new.laps {
        for lap in laps {
            let prefix = format!("lap {}", lap.lap_number);
            check_f64_range(
                &format!("{prefix} distance_km"),
                Some(lap.distance_km),
                &limits.distance_km,
                &mut errors,
            );
            check_i32_range(
                &format!("{prefix} duration_seconds"),
                Some(lap.duration_seconds as i32),
                &limits.duration_seconds,
                &mut errors,
            );
            check_f64_range(
                &format!("{prefix} pace_min_per_km"),
                Some(lap.pace_min_per_km),
                &limits.pace_min_per_km,
                &mut errors,
            );
            check_f64_range(
                &format!("{prefix} avg_heart_rate_bpm"),
                lap.avg_heart_rate_bpm,
                &limits.heart_rate_bpm,
                &mut errors,
            );
            check_f64_range(
                &format!("{prefix} max_heart_rate_bpm"),
                lap.max_heart_rate_bpm,
                &limits.heart_rate_bpm,
                &mut errors,
            );
            if let (Some(avg), Some(max)) = (lap.avg_heart_rate_bpm, lap.max_heart_rate_bpm) {
                if avg.is_finite() && max.is_finite() && avg > max {
                    errors.push(format!(
                        "{prefix}: avg_heart_rate_bpm ({}) must be <= max_heart_rate_bpm ({})",
                        format_num(avg),
                        format_num(max)
                    ));
                }
            }
        }
    }

    finish(errors)
}

pub fn validate_workout_metrics(new: &ProposedWorkoutMetrics, limits: &SanityLimits) -> Result<()> {
    let mut errors = Vec::new();
    check_i32_range(
        "duration_minutes",
        new.duration_minutes,
        &limits.duration_minutes,
        &mut errors,
    );
    check_i32_range(
        "overall_feeling",
        new.overall_feeling,
        &limits.overall_feeling,
        &mut errors,
    );
    finish(errors)
}

/// Validate track samples; fail-fast after collecting up to `max_errors`.
pub fn validate_trackpoints(
    points: &[Trackpoint],
    limits: &SanityLimits,
    max_errors: usize,
) -> Result<()> {
    let mut errors = Vec::new();
    for (i, p) in points.iter().enumerate() {
        if errors.len() >= max_errors {
            errors.push(format!(
                "... and more trackpoint issues (stopped after {} errors)",
                max_errors
            ));
            break;
        }
        let n = i + 1;
        check_f64_range(
            &format!("trackpoint[{n}] latitude"),
            p.latitude,
            &limits.latitude,
            &mut errors,
        );
        check_f64_range(
            &format!("trackpoint[{n}] longitude"),
            p.longitude,
            &limits.longitude,
            &mut errors,
        );
        // GPS altitude may be below sea level; use a wide absolute window.
        check_f64_range(
            &format!("trackpoint[{n}] altitude_m"),
            p.altitude_m,
            &AbsoluteLimits::new(-500.0, 9000.0),
            &mut errors,
        );
        check_f64_range(
            &format!("trackpoint[{n}] heart_rate_bpm"),
            p.heart_rate_bpm,
            &limits.heart_rate_bpm,
            &mut errors,
        );
        check_f64_range(
            &format!("trackpoint[{n}] cadence_spm"),
            p.cadence_spm,
            &limits.cadence_spm,
            &mut errors,
        );
        // Cumulative distance may start at 0.
        check_f64_range(
            &format!("trackpoint[{n}] distance_km"),
            p.distance_km,
            &AbsoluteLimits::new(0.0, limits.distance_km.max),
            &mut errors,
        );
        check_f64_range(
            &format!("trackpoint[{n}] speed_m_s"),
            p.speed_m_s,
            &limits.speed_m_s,
            &mut errors,
        );
    }
    finish(errors)
}

fn check_zone_seconds(field: &str, value: u32, limits: &SanityLimits, errors: &mut Vec<String>) {
    check_i32_range(field, Some(value as i32), &limits.hr_zone_seconds, errors);
}

fn check_f64_range(
    field: &str,
    value: Option<f64>,
    limits: &AbsoluteLimits,
    errors: &mut Vec<String>,
) {
    let Some(v) = value else {
        return;
    };
    if !v.is_finite() {
        errors.push(format!("{} must be a finite number", field));
        return;
    }
    if v < limits.min || v > limits.max {
        errors.push(format!(
            "{} {} is outside allowed range {}–{}",
            field,
            format_num(v),
            format_num(limits.min),
            format_num(limits.max)
        ));
    }
}

fn check_i32_range(
    field: &str,
    value: Option<i32>,
    limits: &AbsoluteLimits,
    errors: &mut Vec<String>,
) {
    let Some(v) = value else {
        return;
    };
    let vf = v as f64;
    if vf < limits.min || vf > limits.max {
        errors.push(format!(
            "{} {} is outside allowed range {}–{}",
            field,
            v,
            format_num(limits.min),
            format_num(limits.max)
        ));
    }
}

fn finish(errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(RepslogError::Cli(errors.join("; ")))
    }
}

fn format_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.3}", v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lim() -> SanityLimits {
        SanityLimits::default()
    }

    #[test]
    fn empty_ok() {
        validate_set_metrics(&ProposedSetMetrics::default(), &lim()).unwrap();
    }

    #[test]
    fn hr_in_range_ok() {
        let m = ProposedSetMetrics {
            avg_heart_rate_bpm: Some(150.0),
            max_heart_rate_bpm: Some(175.0),
            distance_km: Some(5.0),
            duration_seconds: Some(1500),
            avg_pace_min_per_km: Some(5.0),
            calories_burned: Some(300),
            ..Default::default()
        };
        validate_set_metrics(&m, &lim()).unwrap();
    }

    #[test]
    fn hr_out_of_range_fails() {
        let m = ProposedSetMetrics {
            avg_heart_rate_bpm: Some(999.0),
            max_heart_rate_bpm: Some(175.0),
            ..Default::default()
        };
        let err = validate_set_metrics(&m, &lim()).unwrap_err();
        assert!(format!("{}", err).contains("avg_heart_rate_bpm"));
    }

    #[test]
    fn avg_gt_max_hr_fails() {
        let m = ProposedSetMetrics {
            avg_heart_rate_bpm: Some(180.0),
            max_heart_rate_bpm: Some(160.0),
            ..Default::default()
        };
        let err = validate_set_metrics(&m, &lim()).unwrap_err();
        assert!(format!("{}", err).contains("must be <="));
    }

    #[test]
    fn nan_fails() {
        let m = ProposedSetMetrics {
            weight_kg: Some(f64::NAN),
            ..Default::default()
        };
        assert!(validate_set_metrics(&m, &lim()).is_err());
    }

    #[test]
    fn effective_reps_gt_reps_fails() {
        let m = ProposedSetMetrics {
            reps: Some(5),
            effective_reps: Some(8),
            ..Default::default()
        };
        assert!(validate_set_metrics(&m, &lim()).is_err());
    }

    #[test]
    fn feeling_out_of_range() {
        let m = ProposedWorkoutMetrics {
            overall_feeling: Some(6),
            ..Default::default()
        };
        assert!(validate_workout_metrics(&m, &lim()).is_err());
    }
}
