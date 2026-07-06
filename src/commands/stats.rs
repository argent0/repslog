use crate::cli::StatsAction;
use crate::error::Result;
use crate::repository::Repository;
use crate::utils::{format_datetime, print_json, print_table};
use serde::Serialize;
use sqlx::Row;

pub async fn handle_stats(action: StatsAction, repo: &Repository, json: bool) -> Result<()> {
    match action {
        StatsAction::Prs { exercise } => {
            let mut query = "SELECT e.name, \
                MAX(CASE WHEN e.load_type = 'body_mass' THEN es.weight_kg + COALESCE(es.external_load_kg, 0) ELSE es.weight_kg END) as max_weight, \
                MAX(es.reps) as max_reps \
                FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id".to_string();
            let exercise_name = if let Some(ref ex) = exercise {
                Some(repo.require_exercise_by_id_or_name(ex).await?.name)
            } else {
                None
            };
            if exercise_name.is_some() {
                query.push_str(" WHERE e.name = ?");
            }
            query.push_str(" GROUP BY e.name");

            let mut q = sqlx::query(&query);
            if let Some(ref name) = exercise_name {
                q = q.bind(name);
            }
            let res = q.fetch_all(&repo.pool).await?;
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
            let mut query = "SELECT e.name, \
                SUM(CASE \
                    WHEN es.weight_kg IS NULL THEN 0.0 \
                    WHEN e.load_type = 'body_mass' THEN (es.weight_kg + COALESCE(es.external_load_kg, 0)) * es.reps \
                    ELSE es.weight_kg * es.reps \
                END) as total_volume, \
                SUM(es.effective_reps) as total_eff_reps \
                FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id JOIN workouts w ON we.workout_id = w.id".to_string();
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
            let exercise_name = if let Some(ref ex) = exercise {
                Some(repo.require_exercise_by_id_or_name(ex).await?.name)
            } else {
                None
            };
            if exercise_name.is_some() {
                query.push_str(" AND e.name = ?");
            }
            query.push_str(" GROUP BY e.name");

            let mut q = sqlx::query(&query);
            if let Some(ref name) = exercise_name {
                q = q.bind(name);
            }
            let res = q.fetch_all(&repo.pool).await?;
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
            let exercise_name = repo.require_exercise_by_id_or_name(&exercise).await?.name;
            let query = "SELECT w.id AS workout_id, w.started_at, w.workout_type, e.name AS exercise_name, e.load_type AS exercise_load_type, \
                         es.set_number, es.reps, es.weight_kg, es.external_load_kg, es.duration_seconds, es.side, es.rir, \
                         es.effective_reps, es.notes \
                         FROM exercise_sets es \
                         JOIN workout_exercises we ON es.workout_exercise_id = we.id \
                         JOIN exercises e ON we.exercise_id = e.id \
                         JOIN workouts w ON we.workout_id = w.id \
                         WHERE e.name = ? AND w.started_at >= date('now', ?) \
                         ORDER BY w.started_at ASC, es.set_number ASC";
            let days_ago = format!("-{} days", days);
            let res = sqlx::query(query)
                .bind(&exercise_name)
                .bind(&days_ago)
                .fetch_all(&repo.pool)
                .await?;
            #[derive(Serialize)]
            struct HistoryEntry {
                workout_id: i64,
                date: String,
                workout_type: Option<String>,
                exercise: String,
                exercise_load_type: String,
                set_number: i32,
                reps: Option<i32>,
                weight_kg: Option<f64>,
                external_load_kg: Option<f64>,
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
                    exercise_load_type: r.get("exercise_load_type"),
                    set_number: r.get("set_number"),
                    reps: r.get("reps"),
                    weight_kg: r.get("weight_kg"),
                    external_load_kg: r.get("external_load_kg"),
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
                    "Set history for '{}' (last {} days):",
                    exercise_name, days
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
                            crate::bodyweight::format_load_display(
                                &e.exercise_load_type,
                                e.weight_kg,
                                e.external_load_kg,
                            ),
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
            let exercise_name = repo.require_exercise_by_id_or_name(&exercise).await?.name;
            let query = "SELECT w.started_at, es.set_number, es.weight_kg, es.external_load_kg, e.load_type AS exercise_load_type, es.reps, es.notes \
                         FROM exercise_sets es \
                         JOIN workout_exercises we ON es.workout_exercise_id = we.id \
                         JOIN exercises e ON we.exercise_id = e.id \
                         JOIN workouts w ON we.workout_id = w.id \
                         WHERE e.name = ? AND es.weight_kg IS NOT NULL \
                         ORDER BY w.started_at ASC, es.set_number ASC";
            let res = sqlx::query(query)
                .bind(&exercise_name)
                .fetch_all(&repo.pool)
                .await?;
            #[derive(Serialize)]
            struct Load {
                date: String,
                set: i32,
                weight_kg: f64,
                external_load_kg: Option<f64>,
                exercise_load_type: String,
                reps: Option<i32>,
                notes: Option<String>,
            }
            let mut loads = Vec::new();
            for r in res {
                loads.push(Load {
                    date: format_datetime(r.get::<String, _>("started_at").as_str()),
                    set: r.get("set_number"),
                    weight_kg: r.get("weight_kg"),
                    external_load_kg: r.get("external_load_kg"),
                    exercise_load_type: r.get("exercise_load_type"),
                    reps: r.get("reps"),
                    notes: r.get("notes"),
                });
            }
            if json {
                print_json(&loads)?;
            } else {
                println!("Load history for '{}':", exercise_name);
                let mut rows = Vec::new();
                for l in &loads {
                    rows.push(vec![
                        l.date.clone(),
                        l.set.to_string(),
                        crate::bodyweight::format_load_display(
                            &l.exercise_load_type,
                            Some(l.weight_kg),
                            l.external_load_kg,
                        ),
                        l.reps.map(|r| r.to_string()).unwrap_or_default(),
                        l.notes.clone().unwrap_or_default(),
                    ]);
                }
                print_table(vec!["Date", "Set", "Load", "Reps", "Notes"], rows);
            }
        }
    }
    Ok(())
}
