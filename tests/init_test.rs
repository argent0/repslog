use repslog::commands::init::handle_init;
use repslog::db::setup_test_db;
use repslog::repository::Repository;
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn test_handle_init() {
    let pool = setup_test_db().await.unwrap();

    // Initializing with handle_init
    handle_init(&pool, false).await.unwrap();

    let repo = Repository::new(pool);
    let exercises = repo.list_exercises(None, None).await.unwrap();

    // Basic exercises should be present
    assert!(exercises.iter().any(|e| e.name == "Pushups"));
    assert!(exercises.iter().any(|e| e.name == "Pullups"));
    assert!(exercises.iter().any(|e| e.name == "Bench Press"));
    assert!(exercises.len() >= 11);
}

/// Helper to get a unique temp db path for testing custom --db
fn temp_db_path() -> PathBuf {
    let mut p = env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("repslog_test_{}.db", nanos));
    p
}

#[tokio::test]
async fn test_setup_db_with_custom_path() {
    let db_path = temp_db_path();
    let path_str = db_path.to_str().unwrap();

    // Cleanup before (in case of prior failure)
    let _ = std::fs::remove_file(&db_path);

    // Use custom db path (like --db flag)
    {
        let pool = repslog::db::setup_db(Some(path_str)).await.unwrap();

        // Run init which applies migrations + seeds (setup_db only creates migrations table)
        handle_init(&pool, false).await.unwrap();

        // File should now exist
        assert!(db_path.exists(), "custom db file should be created");

        let repo = Repository::new(pool);
        let exercises = repo.list_exercises(None, None).await.unwrap();
        assert!(exercises.iter().any(|e| e.name == "Pushups"));
        assert!(exercises.len() >= 11);
        // pool and repo dropped here
    }

    // Cleanup
    let _ = std::fs::remove_file(path_str);
}
