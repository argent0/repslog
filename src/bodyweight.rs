use crate::error::{RepslogError, Result};
use crate::models::Exercise;

pub const NO_WEIGHT_WARNING: &str = "Warning: body weight not recorded for this set. Volume stats and load history will exclude it. Prefer --weight <kg> with your body mass in kg.";

pub fn is_bodyweight(exercise: &Exercise) -> bool {
    exercise
        .equipment
        .as_deref()
        .is_some_and(|e| e.eq_ignore_ascii_case("bodyweight"))
}

pub fn is_strength_metric_set(
    reps: Option<i32>,
    weight: Option<f64>,
    duration: Option<i32>,
    external_load: Option<f64>,
) -> bool {
    reps.is_some() || weight.is_some() || duration.is_some() || external_load.is_some()
}

pub fn validate_external_load_for_equipment(
    exercise: &Exercise,
    external_load: Option<f64>,
) -> Result<()> {
    if external_load.is_some() && !is_bodyweight(exercise) {
        return Err(RepslogError::Cli(
            "--external-load is only valid for bodyweight exercises (equipment=bodyweight). \
             Use --weight for barbell/dumbbell load."
                .into(),
        ));
    }
    Ok(())
}

pub fn resolve_bodyweight_load(
    exercise: &Exercise,
    weight: Option<f64>,
    external_load: Option<f64>,
    no_weight_recorded: bool,
    requires_body_weight: bool,
) -> Result<(Option<f64>, Option<f64>)> {
    validate_external_load_for_equipment(exercise, external_load)?;

    if no_weight_recorded && weight.is_some() {
        return Err(RepslogError::Cli(
            "Cannot use --weight together with --no-weight-recorded.".into(),
        ));
    }

    if requires_body_weight && is_bodyweight(exercise) {
        if !no_weight_recorded && weight.is_none() {
            return Err(RepslogError::Cli(format!(
                "Bodyweight exercise '{}' requires --weight <kg> (your body mass) \
                 or --no-weight-recorded (not recommended; excludes set from volume stats).",
                exercise.name
            )));
        }
        if let Some(w) = weight {
            if w <= 0.0 {
                return Err(RepslogError::Cli(
                    "Body weight must be a positive value in kg.".into(),
                ));
            }
        }
        if no_weight_recorded {
            eprintln!("{}", NO_WEIGHT_WARNING);
            Ok((None, external_load))
        } else {
            Ok((weight, external_load))
        }
    } else {
        if no_weight_recorded {
            return Err(RepslogError::Cli(
                "--no-weight-recorded is only valid for bodyweight exercises.".into(),
            ));
        }
        Ok((weight, external_load))
    }
}

pub fn total_load_kg(
    equipment: Option<&str>,
    weight_kg: Option<f64>,
    external_load_kg: Option<f64>,
) -> Option<f64> {
    let bodyweight = equipment.is_some_and(|e| e.eq_ignore_ascii_case("bodyweight"));
    match (bodyweight, weight_kg) {
        (true, Some(w)) => Some(w + external_load_kg.unwrap_or(0.0)),
        (false, Some(w)) => Some(w),
        _ => None,
    }
}

pub fn format_load_display(
    equipment: Option<&str>,
    weight_kg: Option<f64>,
    external_load_kg: Option<f64>,
) -> String {
    let bodyweight = equipment.is_some_and(|e| e.eq_ignore_ascii_case("bodyweight"));
    if bodyweight {
        match weight_kg {
            Some(w) => {
                let mut s = format!("{:.1} kg BW", w);
                if let Some(ext) = external_load_kg {
                    if ext.abs() > f64::EPSILON {
                        if ext > 0.0 {
                            s.push_str(&format!(" +{:.1} kg", ext));
                        } else {
                            s.push_str(&format!(" {:.1} kg assist", ext));
                        }
                    }
                }
                s
            }
            None => "(body weight not recorded)".to_string(),
        }
    } else if let Some(w) = weight_kg {
        format!("{:.2} kg", w)
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Exercise;

    fn ex(equipment: Option<&str>) -> Exercise {
        Exercise {
            id: 1,
            name: "pull up".into(),
            category: "calisthenics".into(),
            muscle_groups: None,
            equipment: equipment.map(str::to_string),
            description: None,
            is_custom: 0,
            created_at: None,
        }
    }

    #[test]
    fn requires_weight_for_bodyweight() {
        let err = resolve_bodyweight_load(&ex(Some("bodyweight")), None, None, false, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("requires --weight"));
    }

    #[test]
    fn total_load_includes_external() {
        assert_eq!(
            total_load_kg(Some("bodyweight"), Some(80.0), Some(5.0)),
            Some(85.0)
        );
    }
}
