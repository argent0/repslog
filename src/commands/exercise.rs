use crate::cli::ExerciseAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::{format_dry_run_id, print_id, print_json, print_table};

pub async fn handle_exercise(action: ExerciseAction, repo: &Repository, json: bool) -> Result<()> {
    match action {
        ExerciseAction::List { search, category } => {
            let exercises = repo.list_exercises(search, category).await?;
            if json {
                print_json(&exercises)?;
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
                print_json(&exercises)?;
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
