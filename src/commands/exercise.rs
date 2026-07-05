use crate::cli::ExerciseAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::{format_datetime_opt, format_dry_run_id, print_id, print_json, print_table};

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
