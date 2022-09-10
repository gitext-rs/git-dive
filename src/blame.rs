use anyhow::Context as _;
use proc_exit::WithCodeResultExt;

pub fn blame(
    args: &crate::args::Args,
    _colored_stdout: bool,
    _colored_stderr: bool,
) -> proc_exit::ExitResult {
    pager::Pager::new().setup();

    let file = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", args.file.display()))
        .with_code(proc_exit::Code::CONFIG_ERR)?;

    println!("{}", file);

    Ok(())
}
