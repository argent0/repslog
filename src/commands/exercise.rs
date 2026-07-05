use crate::cli::ExerciseAction;
use crate::error::{RepslogError, Result};
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
                    ]);
                }
                print_table(vec!["ID", "Name", "Category", "Muscles", "Equipment"], rows);
            }
        }
        ExerciseAction::Add {
            name,
            category,
            equipment,
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

            let id = repo
                .add_exercise(
                    &name,
                    &category,
                    muscles.as_deref(),
                    equipment.as_deref(),
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
        ExerciseAction::Search { term } => {
            let exercises = repo.list_exercises(Some(term), None).await?;
            if json {
                #[derive(serde::Serialize)]
                struct ExerciseOut {
                    id: i64,
                    name: String,
                    category: String,
                    created_at: Option<String>,
                }
                let outs: Vec<ExerciseOut> = exercises
                    .into_iter()
                    .map(|ex| ExerciseOut {
                        id: ex.id,
                        name: ex.name,
                        category: ex.category,
                        created_at: format_datetime_opt(&ex.created_at),
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let mut rows = Vec::new();
                for ex in exercises {
                    rows.push(vec![ex.id.to_string(), ex.name, ex.category]);
                }
                print_table(vec!["ID", "Name", "Category"], rows);
            }
        }
    }
    Ok(())
}
