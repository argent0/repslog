use crate::error::{RepslogError, Result};
use crate::load_type;
use crate::models::Exercise;

pub const NO_WEIGHT_WARNING: &str = "Warning: body weight not recorded for this set. Volume stats and load history will exclude it. Prefer --weight <kg> with your body mass in kg.";

pub fn uses_body_mass(exercise: &Exercise) -> bool {
    load_type::is_body_mass(&exercise.load_type)
}

pub fn is_strength_metric_set(
    reps: Option<i32>,
    weight: Option<f64>,
    duration: Option<i32>,
    external_load: Option<f64>,
) -> bool {
    reps.is_some() || weight.is_some() || duration.is_some() || external_load.is_some()
}

pub fn validate_external_load(load_type: &str, external_load: Option<f64>) -> Result<()> {
    if external_load.is_some() && !load_type::is_body_mass(load_type) {
        return Err(RepslogError::Cli(
            "--external-load is only valid for body-mass exercises (load_type=body_mass). \
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
    validate_external_load(&exercise.load_type, external_load)?;

    if no_weight_recorded && weight.is_some() {
        return Err(RepslogError::Cli(
            "Cannot use --weight together with --no-weight-recorded.".into(),
        ));
    }

    if requires_body_weight && uses_body_mass(exercise) {
        if !no_weight_recorded && weight.is_none() {
            return Err(RepslogError::Cli(format!(
                "Exercise '{}' (load_type=body_mass) requires --weight <kg> (your body mass) \
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
                "--no-weight-recorded is only valid for body-mass exercises (load_type=body_mass)."
                    .into(),
            ));
        }
        Ok((weight, external_load))
    }
}

pub fn total_load_kg(
    load_type: &str,
    weight_kg: Option<f64>,
    external_load_kg: Option<f64>,
) -> Option<f64> {
    match (load_type::is_body_mass(load_type), weight_kg) {
        (true, Some(w)) => Some(w + external_load_kg.unwrap_or(0.0)),
        (false, Some(w)) => Some(w),
        _ => None,
    }
}

pub fn format_load_display(
    load_type: &str,
    weight_kg: Option<f64>,
    external_load_kg: Option<f64>,
) -> String {
    if load_type::is_body_mass(load_type) {
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
    use crate::load_type::BODY_MASS;
    use crate::models::Exercise;

    fn ex(load_type: &str) -> Exercise {
        Exercise {
            id: 1,
            name: "pull up".into(),
            category: "calisthenics".into(),
            muscle_groups: None,
            equipment: Some("rings".into()),
            load_type: load_type.to_string(),
            description: None,
            is_custom: 0,
            created_at: None,
        }
    }

    #[test]
    fn requires_weight_for_body_mass() {
        let err = resolve_bodyweight_load(&ex(BODY_MASS), None, None, false, true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("requires --weight"));
    }

    #[test]
    fn total_load_includes_external() {
        assert_eq!(total_load_kg(BODY_MASS, Some(80.0), Some(5.0)), Some(85.0));
    }

    #[test]
    fn rings_with_body_mass_load_type_requires_weight() {
        let exercise = ex(BODY_MASS);
        assert!(uses_body_mass(&exercise));
        assert!(resolve_bodyweight_load(&exercise, Some(80.0), None, false, true).is_ok());
    }
}
