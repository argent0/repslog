use repslog::commands::init::handle_init;
use repslog::db::setup_test_db;
use repslog::repository::Repository;

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
