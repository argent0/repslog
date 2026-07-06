use crate::cli::ExerciseAction;
use crate::error::{RepslogError, Result};
use crate::load_type::{self, normalize_load_type};
use crate::repository::Repository;
use crate::utils::{
    find_exercise_name_conflicts, format_datetime_opt, format_dry_run_id, normalize_exercise_name,
    print_id, print_json, print_table, suggest_singular_exercise_name, ExerciseNameConflictKind,
};

pub async fn handle_exercise(action: ExerciseAction, repo: &Repository, json: bool) -> Result<()> {
    match action {
        ExerciseAction::List { search, category } => {
            let exercises = repo.list_exercises(search, category).await?;
            if json {
                #[derive(serde::Serialize)]
                struct ExerciseOut {
                    id: i64,
                    name: String,
                    category: String,
                    muscle_groups: Option<String>,
                    equipment: Option<String>,
                    load_type: String,
                    description: Option<String>,
                    is_custom: i32,
                    created_at: Option<String>,
                }
                let outs: Vec<ExerciseOut> = exercises
                    .into_iter()
                    .map(|ex| ExerciseOut {
                        id: ex.id,
                        name: ex.name,
                        category: ex.category,
                        muscle_groups: ex.muscle_groups,
                        equipment: ex.equipment,
                        load_type: ex.load_type,
                        description: ex.description,
                        is_custom: ex.is_custom,
                        created_at: format_datetime_opt(&ex.created_at),
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let mut rows = Vec::new();
                for ex in exercises {
                    rows.push(vec![
                        ex.id.to_string(),
                        ex.name,
                        ex.category,
                        ex.muscle_groups.unwrap_or_default(),
                        ex.equipment.unwrap_or_default(),
                        ex.load_type,
                    ]);
                }
                print_table(
                    vec![
                        "ID",
                        "Name",
                        "Category",
                        "Muscles",
                        "Equipment",
                        "Load Type",
                    ],
                    rows,
                );
            }
        }
        ExerciseAction::Add {
            name,
            category,
            equipment,
            load_type,
            muscles,
            description,
            dry_run,
        } => {
            let name = normalize_exercise_name(&name)?;
            if let Some(singular) = suggest_singular_exercise_name(&name) {
                eprintln!(
                    "Warning: Prefer singular exercise names (e.g. '{}' instead of '{}').",
                    singular, name
                );
            }
            let existing = repo.list_exercises(None, None).await?;
            let catalog: Vec<(i64, String)> =
                existing.iter().map(|ex| (ex.id, ex.name.clone())).collect();
            let conflicts = find_exercise_name_conflicts(&name, &catalog);
            for conflict in &conflicts {
                match conflict.kind {
                    ExerciseNameConflictKind::Duplicate => {
                        return Err(RepslogError::Cli(format!(
                            "Exercise already exists as '{}' (id: {}). \
                             Use `repslog exercise search` to find existing entries.",
                            conflict.existing_name, conflict.existing_id
                        )));
                    }
                    ExerciseNameConflictKind::Similar => {
                        eprintln!(
                            "Warning: '{}' is similar to existing '{}' (id: {}). \
                             Prefer the existing entry to avoid fragmenting history.",
                            name, conflict.existing_name, conflict.existing_id
                        );
                    }
                }
            }

            let (resolved_load_type, resolved_equipment, deprecated_bodyweight_equipment) =
                load_type::resolve_for_new_exercise(
                    &category,
                    equipment.as_deref(),
                    load_type.as_deref(),
                )?;
            if deprecated_bodyweight_equipment {
                eprintln!(
                    "Warning: --equipment bodyweight is deprecated. \
                     Use --load-type body_mass for body-mass tracking and --equipment for apparatus (rings, barbell, etc.)."
                );
            }

            let id = repo
                .add_exercise(
                    &name,
                    &category,
                    muscles.as_deref(),
                    resolved_equipment.as_deref(),
                    &resolved_load_type,
                    description.as_deref(),
                    true,
                    dry_run,
                )
                .await?;
            let formatted_id = format_dry_run_id(id, dry_run);
            if json {
                print_id(&formatted_id, true);
            } else {
                eprintln!("Added exercise {} with ID {}", name, formatted_id);
                println!("{}", formatted_id);
            }
        }
        ExerciseAction::Update {
            exercise_id_or_name,
            category,
            equipment,
            clear_equipment,
            load_type,
            muscles,
            description,
            dry_run,
        } => {
            if clear_equipment && equipment.is_some() {
                return Err(RepslogError::Cli(
                    "Cannot use --equipment together with --clear-equipment.".into(),
                ));
            }
            if category.is_none()
                && equipment.is_none()
                && !clear_equipment
                && load_type.is_none()
                && muscles.is_none()
                && description.is_none()
            {
                return Err(RepslogError::Cli(
                    "At least one field to update is required.".into(),
                ));
            }

            let exercise = repo
                .find_exercise_by_id_or_name(&exercise_id_or_name)
                .await?
                .ok_or_else(|| {
                    RepslogError::Cli(format!("Exercise '{}' not found", exercise_id_or_name))
                })?;

            let resolved_load_type = if let Some(value) = load_type.as_deref() {
                Some(normalize_load_type(value)?.to_string())
            } else {
                None
            };
            let resolved_equipment = if clear_equipment {
                Some(None)
            } else {
                equipment.as_deref().map(|value| Some(value.to_string()))
            };

            repo.update_exercise(
                exercise.id,
                category.as_deref(),
                resolved_equipment.as_ref().map(|value| value.as_deref()),
                resolved_load_type.as_deref(),
                muscles.as_deref(),
                description.as_deref(),
                dry_run,
            )
            .await?;

            if json {
                #[derive(serde::Serialize)]
                struct UpdateOut {
                    id: i64,
                    name: String,
                    dry_run: bool,
                }
                print_json(&UpdateOut {
                    id: exercise.id,
                    name: exercise.name,
                    dry_run,
                })?;
            } else if dry_run {
                eprintln!(
                    "Dry run: would update exercise '{}' (id: {})",
                    exercise.name, exercise.id
                );
            } else {
                eprintln!("Updated exercise '{}' (id: {})", exercise.name, exercise.id);
            }
        }
        ExerciseAction::Search { term } => {
            let exercises = repo.list_exercises(Some(term), None).await?;
            if json {
                #[derive(serde::Serialize)]
                struct ExerciseOut {
                    id: i64,
                    name: String,
                    category: String,
                    load_type: String,
                    created_at: Option<String>,
                }
                let outs: Vec<ExerciseOut> = exercises
                    .into_iter()
                    .map(|ex| ExerciseOut {
                        id: ex.id,
                        name: ex.name,
                        category: ex.category,
                        load_type: ex.load_type,
                        created_at: format_datetime_opt(&ex.created_at),
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let mut rows = Vec::new();
                for ex in exercises {
                    rows.push(vec![ex.id.to_string(), ex.name, ex.category, ex.load_type]);
                }
                print_table(vec!["ID", "Name", "Category", "Load Type"], rows);
            }
        }
    }
    Ok(())
}
