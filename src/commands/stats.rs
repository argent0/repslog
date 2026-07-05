use crate::cli::StatsAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::{format_datetime, print_json, print_table};
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
        StatsAction::History { exercise, days } => {
            let query = "SELECT w.id AS workout_id, w.started_at, w.workout_type, e.name AS exercise_name, \
                         es.set_number, es.reps, es.weight_kg, es.duration_seconds, es.side, es.rir, \
                         es.effective_reps, es.notes \
                         FROM exercise_sets es \
                         JOIN workout_exercises we ON es.workout_exercise_id = we.id \
                         JOIN exercises e ON we.exercise_id = e.id \
                         JOIN workouts w ON we.workout_id = w.id \
                         WHERE e.name LIKE ? AND w.started_at >= date('now', ?) \
                         ORDER BY w.started_at ASC, es.set_number ASC";
            let like = format!("%{}%", exercise);
            let days_ago = format!("-{} days", days);
            let res = sqlx::query(query)
                .bind(&like)
                .bind(&days_ago)
                .fetch_all(&repo.pool)
                .await?;
            #[derive(Serialize)]
            struct HistoryEntry {
                workout_id: i64,
                date: String,
                workout_type: Option<String>,
                exercise: String,
                set_number: i32,
                reps: Option<i32>,
                weight_kg: Option<f64>,
                duration_seconds: Option<i32>,
                side: Option<String>,
                rir: Option<f64>,
                effective_reps: Option<i32>,
                notes: Option<String>,
            }
            let mut entries = Vec::new();
            for r in res {
                entries.push(HistoryEntry {
                    workout_id: r.get("workout_id"),
                    date: format_datetime(r.get::<String, _>("started_at").as_str()),
                    workout_type: r.get("workout_type"),
                    exercise: r.get("exercise_name"),
                    set_number: r.get("set_number"),
                    reps: r.get("reps"),
                    weight_kg: r.get("weight_kg"),
                    duration_seconds: r.get("duration_seconds"),
                    side: r.get("side"),
                    rir: r.get("rir"),
                    effective_reps: r.get("effective_reps"),
                    notes: r.get("notes"),
                });
            }
            if json {
                print_json(&entries)?;
            } else {
                println!(
                    "Set history for exercises matching '{}' (last {} days):",
                    exercise, days
                );
                if entries.is_empty() {
                    println!("No sets found in this period.");
                } else {
                    let mut rows = Vec::new();
                    for e in &entries {
                        rows.push(vec![
                            e.date.clone(),
                            e.workout_id.to_string(),
                            e.set_number.to_string(),
                            e.reps.map(|r| r.to_string()).unwrap_or_else(|| {
                                e.duration_seconds
                                    .map(|d| format!("{}s", d))
                                    .unwrap_or_default()
                            }),
                            e.weight_kg
                                .map(|w| format!("{:.2} kg", w))
                                .unwrap_or_default(),
                            e.side.clone().unwrap_or_default(),
                            e.notes.clone().unwrap_or_default(),
                        ]);
                    }
                    print_table(
                        vec!["Date", "Workout", "Set", "Reps", "Weight", "Side", "Notes"],
                        rows,
                    );
                }
            }
        }
        StatsAction::Weight { exercise } => {
            // Basic weight progression: join to get workout date + sets with weight for the exercise
            let query = "SELECT w.started_at, es.set_number, es.weight_kg, es.reps, es.notes \
                         FROM exercise_sets es \
                         JOIN workout_exercises we ON es.workout_exercise_id = we.id \
                         JOIN exercises e ON we.exercise_id = e.id \
                         JOIN workouts w ON we.workout_id = w.id \
                         WHERE e.name LIKE ? AND es.weight_kg IS NOT NULL \
                         ORDER BY w.started_at ASC, es.set_number ASC";
            let like = format!("%{}%", exercise);
            let res = sqlx::query(query).bind(&like).fetch_all(&repo.pool).await?;
            #[derive(Serialize)]
            struct Load {
                date: String,
                set: i32,
                weight_kg: f64,
                reps: Option<i32>,
                notes: Option<String>,
            }
            let mut loads = Vec::new();
            for r in res {
                loads.push(Load {
                    date: format_datetime(r.get::<String, _>("started_at").as_str()),
                    set: r.get("set_number"),
                    weight_kg: r.get("weight_kg"),
                    reps: r.get("reps"),
                    notes: r.get("notes"),
                });
            }
            if json {
                print_json(&loads)?;
            } else {
                println!("Weight history for exercises matching '{}':", exercise);
                let mut rows = Vec::new();
                for l in &loads {
                    rows.push(vec![
                        l.date.clone(),
                        l.set.to_string(),
                        format!("{:.2} kg", l.weight_kg),
                        l.reps.map(|r| r.to_string()).unwrap_or_default(),
                        l.notes.clone().unwrap_or_default(),
                    ]);
                }
                print_table(vec!["Date", "Set", "Weight", "Reps", "Notes"], rows);
            }
        }
    }
    Ok(())
}
