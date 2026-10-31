//! Invalid fixture with shell execution string
pub fn run_bad() {
    let _cmd = ("bash", "-c", "echo hello");
}
