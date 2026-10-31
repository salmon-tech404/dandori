//! Invalid fixture using format! in SQL query
pub async fn bad_query(pool: &sqlx::SqlitePool, id: &str) {
    let _ = sqlx::query(&format!("SELECT * FROM users WHERE id = '{}'", id))
        .execute(pool)
        .await;
}
