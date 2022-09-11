use std::io::Write;

use anyhow::Context as _;
use proc_exit::WithCodeResultExt;

pub fn blame(
    args: &crate::args::Args,
    colored_stdout: bool,
    _colored_stderr: bool,
) -> proc_exit::ExitResult {
    let total_width = terminal_size::terminal_size()
        .map(|(w, _h)| w.0)
        .or_else(|| std::env::var_os("COLUMNS").and_then(|s| s.to_str()?.parse::<u16>().ok()))
        .unwrap_or(80) as usize;

    #[cfg(unix)]
    pager::Pager::new().setup();

    let repo = git2::Repository::discover(".").with_code(proc_exit::Code::CONFIG_ERR)?;
    let config = repo.config().with_code(proc_exit::Code::CONFIG_ERR)?;
    let theme = config.get_string(THEME_FIELD).ok();

    let mut file = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", args.file.display()))
        .with_code(proc_exit::Code::CONFIG_ERR)?;

    let line_count = file.lines().count();
    let line_count_width = line_count.to_string().len(); // bytes = chars = columns with digits
    let sep = " │ ";

    let code_width = total_width - line_count_width - sep.len();

    if let Some(ext) = colored_stdout
        .then(|| args.file.extension().and_then(|s| s.to_str()))
        .flatten()
    {
        file = highlight(&file, ext, theme.as_deref()).with_code(proc_exit::Code::UNKNOWN)?;
    }

    let mut stdout = std::io::stdout().lock();
    let reset = anstyle::Reset.render();
    let wrap = textwrap::Options::new(code_width)
        .break_words(false)
        .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit);

    for (line_num, file_line) in file.lines().enumerate() {
        let line_num = line_num + 1;
        for (i, visual_line) in textwrap::wrap(file_line, &wrap).into_iter().enumerate() {
            if i == 0 {
                let _ = writeln!(
                    &mut stdout,
                    "{reset}{line_num:>line_count_width$}{sep}{visual_line}"
                );
            }
        }
    }

    Ok(())
}

fn highlight(file: &str, ext: &str, theme: Option<&str>) -> anyhow::Result<String> {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::SyntaxSet;
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    let mut output = String::new();

    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let theme = theme.unwrap_or(THEME_DEFAULT);
    let theme = ts
        .themes
        .get(theme)
        .or_else(|| ts.themes.get(THEME_DEFAULT))
        .expect("default theme is present");

    let syntax = match ps.find_syntax_by_extension(ext) {
        Some(syntax) => syntax,
        None => {
            return Ok(file.to_owned());
        }
    };
    let mut h = HighlightLines::new(syntax, theme);
    for line in LinesWithEndings::from(file) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps)?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        output.push_str(&escaped);
    }

    Ok(output)
}

const THEME_FIELD: &str = "dive.theme";
const THEME_DEFAULT: &str = "base16-ocean.dark";
