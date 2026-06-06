use crate::cli::StatsAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::{print_json, print_table};
use serde::Serialize;
use sqlx::Row;

pub async fn handle_stats(action: StatsAction, repo: &Repository, json: bool) -> Result<()> {
    match action {
        StatsAction::Prs { exercise } => {
            let mut query = "SELECT e.name, MAX(es.weight_kg) as max_weight, MAX(es.reps) as max_reps FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id".to_string();
            if let Some(ex) = exercise {
                query.push_str(&format!(" WHERE e.name LIKE '%{}%'", ex));
            }
            query.push_str(" GROUP BY e.name");

            let res = sqlx::query(&query).fetch_all(&repo.pool).await?;
            #[derive(Serialize)]
            struct Pr {
                exercise: String,
                max_weight: Option<f64>,
                max_reps: Option<i32>,
            }
            let mut prs = Vec::new();
            for r in res {
                let name: String = r.get("name");
                let max_weight: Option<f64> = r.get("max_weight");
                let max_reps: Option<i32> = r.get("max_reps");
                prs.push(Pr {
                    exercise: name,
                    max_weight,
                    max_reps,
                });
            }
            if json {
                print_json(&prs)?;
            } else {
                println!("Personal Records:");
                let mut rows = Vec::new();
                for pr in &prs {
                    rows.push(vec![
                        pr.exercise.clone(),
                        pr.max_weight
                            .map(|w| format!("{:.2} kg", w))
                            .unwrap_or_default(),
                        pr.max_reps.map(|r| r.to_string()).unwrap_or_default(),
                    ]);
                }
                print_table(vec!["Exercise", "Max Weight", "Max Reps"], rows);
            }
        }
        StatsAction::Volume { exercise, period } => {
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
            #[derive(Serialize)]
            struct Vol {
                exercise: String,
                total_volume: Option<f64>,
                total_eff_reps: Option<i64>,
            }
            let mut vols = Vec::new();
            for r in res {
                let name: String = r.get("name");
                let volume: Option<f64> = r.get("total_volume");
                let eff_reps: Option<i64> = r.get("total_eff_reps");
                vols.push(Vol {
                    exercise: name,
                    total_volume: volume,
                    total_eff_reps: eff_reps,
                });
            }
            if json {
                print_json(&vols)?;
            } else {
                println!("Training Volume for period: {}", period);
                let mut rows = Vec::new();
                for v in &vols {
                    rows.push(vec![
                        v.exercise.clone(),
                        v.total_volume
                            .map(|v| format!("{:.2} kg", v))
                            .unwrap_or_default(),
                        v.total_eff_reps.map(|r| r.to_string()).unwrap_or_default(),
                    ]);
                }
                print_table(
                    vec!["Exercise", "Total Volume (kg * reps)", "Total Eff Reps"],
                    rows,
                );
            }
        }
        StatsAction::Summary { days } => {
            let workouts = repo.list_workouts(100, Some(days)).await?;
            let count = workouts.len();
            let total_min: i32 = workouts.iter().filter_map(|w| w.duration_minutes).sum();
            let avg = if count > 0 {
                total_min / count as i32
            } else {
                0
            };
            if json {
                #[derive(Serialize)]
                struct Sum {
                    days: i64,
                    total_workouts: usize,
                    total_duration_minutes: i32,
                    average_duration_minutes: i32,
                }
                print_json(&Sum {
                    days,
                    total_workouts: count,
                    total_duration_minutes: total_min,
                    average_duration_minutes: avg,
                })?;
            } else {
                println!("Summary for last {} days:", days);
                println!("Total Workouts: {}", count);
                println!("Total Duration: {} min", total_min);
                if count > 0 {
                    println!("Average Duration: {} min", avg);
                }
            }
        }
    }
    Ok(())
}
