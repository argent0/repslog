use crate::cli::ExerciseAction;
use crate::repository::Repository;
use crate::error::Result;
use crate::utils::print_table;

pub async fn handle_exercise(action: ExerciseAction, repo: &Repository) -> Result<()> {
    match action {
        ExerciseAction::List { search, category } => {
            let exercises = repo.list_exercises(search, category).await?;
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
        ExerciseAction::Add { name, category, equipment, muscles, description } => {
            let id = repo.add_exercise(&name, &category, muscles.as_deref(), equipment.as_deref(), description.as_deref(), true).await?;
            println!("Added exercise {} with ID {}", name, id);
        }
        ExerciseAction::Search { term } => {
            let exercises = repo.list_exercises(Some(term), None).await?;
            let mut rows = Vec::new();
            for ex in exercises {
                rows.push(vec![
                    ex.id.to_string(),
                    ex.name,
                    ex.category,
                ]);
            }
            print_table(vec!["ID", "Name", "Category"], rows);
        }
    }
    Ok(())
}
