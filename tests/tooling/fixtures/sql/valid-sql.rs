//! Valid fixture using bound parameters
pub async fn good_query(pool: &sqlx::SqlitePool, id: &str) {
    let _ = sqlx::query!("SELECT id FROM users WHERE id = $1", id)
        .fetch_one(pool)
        .await;
}
