#[test]
#[cfg(unix)]
fn screenshot() {
    use term_transcript::{test::TestConfig, ShellOptions};

    let scratchpad = snapbox::path::PathFixture::mutable_temp().unwrap();
    let scratchpad_path = scratchpad.path().unwrap();
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg(std::env::current_dir().unwrap())
        .current_dir(scratchpad_path)
        .status()
        .unwrap();
    assert!(status.success());
    let repo_path = scratchpad_path.join("git-dive");
    let status = std::process::Command::new("git")
        .arg("checkout")
        .arg("ae51fc8be9e4ec83d47a6d83c80d015212a396a5")
        .current_dir(&repo_path)
        .status()
        .unwrap();
    assert!(status.success());

    let cmd_path = snapbox::cmd::cargo_bin("git-dive");

    // HACK: term_transcript doesn't allow non-UTF8 paths
    let cmd = "git-dive Cargo.toml";

    TestConfig::new(
        ShellOptions::<term_transcript::StdShell>::sh()
            .with_alias("git-dive", &cmd_path.to_string_lossy())
            .with_current_dir(&repo_path)
            .with_env("CLICOLOR_FORCE", "1"),
    )
    .test("assets/screenshot.svg", [cmd]);

    scratchpad.close().unwrap();
}
