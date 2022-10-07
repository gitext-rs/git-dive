use proc_exit::WithCodeResultExt;

use crate::git2_config::Config;
use crate::git2_config::ReflectField;

pub fn dump_config(output_path: &std::path::Path, config: &mut Config) -> proc_exit::ExitResult {
    let cwd = std::env::current_dir().with_code(proc_exit::Code::FAILURE)?;
    let repo = git2::Repository::discover(&cwd).with_code(proc_exit::Code::FAILURE)?;

    config.add_repo(&repo);
    let output = config.dump([
        &crate::git2_config::COLOR_UI as &dyn ReflectField,
        &crate::git2_config::PAGER as &dyn ReflectField,
        &crate::blame::THEME as &dyn ReflectField,
    ]);

    if output_path == std::path::Path::new("-") {
        use std::io::Write;
        std::io::stdout()
            .write_all(output.as_bytes())
            .with_code(proc_exit::Code::FAILURE)?;
    } else {
        std::fs::write(output_path, &output).with_code(proc_exit::Code::FAILURE)?;
    }

    Ok(())
}
