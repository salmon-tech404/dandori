fn bad_spawn() {
    let _ = std::process::Command::new("git").arg("status").spawn();
}
