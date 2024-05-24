use snapbox::prelude::*;

#[test]
fn basic() {
    let root = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let root_path = root.path().unwrap();
    let plan = git_fixture::TodoList {
        commands: vec![
            git_fixture::Command::Tree(git_fixture::Tree {
                files: [("basic.js", "test('arg1');")]
                    .into_iter()
                    .map(|(p, c)| (p.into(), c.into()))
                    .collect::<std::collections::HashMap<_, _>>(),
                message: Some("A".to_owned()),
                author: None,
            }),
            git_fixture::Command::Branch("main".into()),
        ],
        ..Default::default()
    };
    plan.run(root_path).unwrap();

    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("git-dive"))
        .arg("basic.js")
        .current_dir(root_path)
        .assert()
        .success()
        .stdout_eq(
            "\
HEAD 1 │ test('arg1');
"
            .raw(),
        )
        .stderr_eq(
            "\
",
        );

    root.close().unwrap();
}

#[test]
fn js_highlight_panics() {
    let root = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let root_path = root.path().unwrap();
    let plan = git_fixture::TodoList {
        commands: vec![
            git_fixture::Command::Tree(git_fixture::Tree {
                files: [("basic.js", "test('arg1');")]
                    .into_iter()
                    .map(|(p, c)| (p.into(), c.into()))
                    .collect::<std::collections::HashMap<_, _>>(),
                message: Some("A".to_owned()),
                author: None,
            }),
            git_fixture::Command::Branch("main".into()),
        ],
        ..Default::default()
    };
    plan.run(root_path).unwrap();

    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("git-dive"))
        .arg("basic.js")
        .current_dir(root_path)
        .env("CLICOLOR_FORCE", "1")
        .assert()
        .success()
        .stderr_eq(
            "\
",
        );

    root.close().unwrap();
}
