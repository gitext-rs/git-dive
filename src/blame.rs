use anyhow::Context as _;
use proc_exit::WithCodeResultExt;

pub fn blame(
    args: &crate::args::Args,
    colored_stdout: bool,
    _colored_stderr: bool,
) -> proc_exit::ExitResult {
    #[cfg(unix)]
    pager::Pager::new().setup();

    let mut file = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", args.file.display()))
        .with_code(proc_exit::Code::CONFIG_ERR)?;

    if let Some(ext) = colored_stdout
        .then(|| args.file.extension().and_then(|s| s.to_str()))
        .flatten()
    {
        file = highlight(&file, ext).with_code(proc_exit::Code::UNKNOWN)?;
    }

    println!("{}", file);

    Ok(())
}

fn highlight(file: &str, ext: &str) -> anyhow::Result<String> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::SyntaxSet;
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    let mut output = String::new();

    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = match ps.find_syntax_by_extension(ext) {
        Some(syntax) => syntax,
        None => {
            return Ok(file.to_owned());
        }
    };
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(file) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps)?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        output.push_str(&escaped);
    }

    Ok(output)
}
