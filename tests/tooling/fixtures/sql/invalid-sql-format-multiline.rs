fn run(x: &str) {
    sqlx::query(
        &format!("select {}", x)
    );
}
