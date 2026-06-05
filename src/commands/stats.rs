use crate::cli::StatsAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::print_table;
use sqlx::Row;

pub async fn handle_stats(action: StatsAction, repo: &Repository) -> Result<()> {
    match action {
        StatsAction::Prs { exercise } => {
            println!("Personal Records:");
            let mut query = "SELECT e.name, MAX(es.weight_kg) as max_weight, MAX(es.reps) as max_reps FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id".to_string();
            if let Some(ex) = exercise {
                query.push_str(&format!(" WHERE e.name LIKE '%{}%'", ex));
            }
            query.push_str(" GROUP BY e.name");

            let res = sqlx::query(&query).fetch_all(&repo.pool).await?;
            let mut rows = Vec::new();
            for r in res {
                let name: String = r.get("name");
                let max_weight: Option<f64> = r.get("max_weight");
                let max_reps: Option<i32> = r.get("max_reps");
                rows.push(vec![
                    name,
                    max_weight
                        .map(|w| format!("{:.2} kg", w))
                        .unwrap_or_default(),
                    max_reps.map(|r| r.to_string()).unwrap_or_default(),
                ]);
            }
            print_table(vec!["Exercise", "Max Weight", "Max Reps"], rows);
        }
        StatsAction::Volume { exercise, period } => {
            println!("Training Volume for period: {}", period);
            let mut query = "SELECT e.name, SUM(es.weight_kg * es.reps) as total_volume, SUM(es.effective_reps) as total_eff_reps FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id JOIN workouts w ON we.workout_id = w.id".to_string();
            let days = match period.as_str() {
                "30d" => 30,
                "90d" => 90,
                "1y" => 365,
                _ => 30,
            };
            query.push_str(&format!(
                " WHERE w.started_at >= date('now', '-{} days')",
                days
            ));
            if let Some(ex) = exercise {
                query.push_str(&format!(" AND e.name LIKE '%{}%'", ex));
            }
            query.push_str(" GROUP BY e.name");

            let res = sqlx::query(&query).fetch_all(&repo.pool).await?;
            let mut rows = Vec::new();
            for r in res {
                let name: String = r.get("name");
                let volume: Option<f64> = r.get("total_volume");
                let eff_reps: Option<i64> = r.get("total_eff_reps");
                rows.push(vec![
                    name,
                    volume.map(|v| format!("{:.2} kg", v)).unwrap_or_default(),
                    eff_reps.map(|r| r.to_string()).unwrap_or_default(),
                ]);
            }
            print_table(
                vec!["Exercise", "Total Volume (kg * reps)", "Total Eff Reps"],
                rows,
            );
        }
        StatsAction::Summary { days } => {
            println!("Summary for last {} days:", days);
            let workouts = repo.list_workouts(100, Some(days)).await?;
            let count = workouts.len();
            let total_min: i32 = workouts.iter().filter_map(|w| w.duration_minutes).sum();
            println!("Total Workouts: {}", count);
            println!("Total Duration: {} min", total_min);
            if count > 0 {
                println!("Average Duration: {} min", total_min / count as i32);
            }
        }
    }
    Ok(())
}
