use std::io::Write;

use anyhow::Context as _;
use encoding::Encoding as _;
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

    let repo = git2::Repository::discover(".").with_code(proc_exit::Code::CONFIG_ERR)?;
    let config = repo.config().with_code(proc_exit::Code::CONFIG_ERR)?;
    let theme = config.get_string(THEME_FIELD).ok();

    let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let theme_set = syntect::highlighting::ThemeSet::load_defaults();
    let theme = theme.as_deref().unwrap_or(THEME_DEFAULT);
    let theme = theme_set
        .themes
        .get(theme)
        .or_else(|| theme_set.themes.get(THEME_DEFAULT))
        .expect("default theme is present");

    let syntax = syntax_set
        .find_syntax_for_file(&args.file)?
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let file = read_file(&args.file).with_code(proc_exit::Code::CONFIG_ERR)?;

    let line_count = file.lines().count();
    let line_count_width = line_count.to_string().len(); // bytes = chars = columns with digits
    let sep = " │ ";

    let code_width = total_width - line_count_width - sep.len();

    let mut highlighter = if colored_stdout {
        Highlighter::enabled(syntax, theme)
    } else {
        Highlighter::disabled()
    };

    #[cfg(unix)]
    pager::Pager::new().setup();

    let mut stdout = std::io::stdout().lock();
    let reset = if colored_stdout {
        anstyle::Reset.render().to_string()
    } else {
        "".to_owned()
    };
    let wrap = textwrap::Options::new(code_width)
        .break_words(false)
        .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit);

    for (line_num, file_line) in file.lines().enumerate() {
        let line_num = line_num + 1;

        let file_line = if line_num == 1 {
            file_line.strip_prefix('\u{feff}').unwrap_or(file_line)
        } else {
            file_line
        };

        let file_line = highlighter
            .highlight_line(file_line, &syntax_set)
            .with_code(proc_exit::Code::UNKNOWN)?;
        for (i, visual_line) in textwrap::wrap(&file_line, &wrap).into_iter().enumerate() {
            if i == 0 {
                let _ = write!(
                    &mut stdout,
                    "{line_num:>line_count_width$}{sep}{visual_line}\n{reset}"
                );
            }
        }
    }

    Ok(())
}

fn read_file(path: &std::path::Path) -> anyhow::Result<String> {
    let buffer =
        std::fs::read(path).with_context(|| format!("Could not read {}", path.display()))?;

    let content_type = content_inspector::inspect(&buffer);

    let buffer = match content_type {
        content_inspector::ContentType::BINARY |
        // HACK: We don't support UTF-32 yet
        content_inspector::ContentType::UTF_32LE |
        content_inspector::ContentType::UTF_32BE => {
            anyhow::bail!("Could not ready binary file {}", path.display())
        },
        content_inspector::ContentType::UTF_8 |
        content_inspector::ContentType::UTF_8_BOM => {
            String::from_utf8_lossy(&buffer).into_owned()
        },
        content_inspector::ContentType::UTF_16LE => {
            let buffer = encoding::all::UTF_16LE.decode(&buffer, encoding::DecoderTrap::Replace)
                .map_err(|_| anyhow::format_err!("Could not decode UTF-16 in {}", path.display()))?;
            buffer
        }
        content_inspector::ContentType::UTF_16BE => {
            let buffer = encoding::all::UTF_16BE.decode(&buffer, encoding::DecoderTrap::Replace)
                .map_err(|_| anyhow::format_err!("Could not decode UTF-16 in {}", path.display()))?;
            buffer
        },
    };

    Ok(buffer)
}

pub struct Highlighter<'a> {
    highlighter: Option<syntect::easy::HighlightLines<'a>>,
}

impl<'a> Highlighter<'a> {
    fn enabled(
        syntax: &'a syntect::parsing::SyntaxReference,
        theme: &'a syntect::highlighting::Theme,
    ) -> Self {
        let highlighter = Some(syntect::easy::HighlightLines::new(syntax, theme));
        Self { highlighter }
    }

    fn disabled() -> Self {
        let highlighter = None;
        Self { highlighter }
    }

    fn highlight_line(
        &mut self,
        line: &str,
        syntax_set: &syntect::parsing::SyntaxSet,
    ) -> anyhow::Result<String> {
        if let Some(highlighter) = &mut self.highlighter {
            // skip syntax highlighting on long lines
            let too_long = line.len() > 1024 * 16;
            let for_highlighting: &str = if too_long { "\n" } else { &line };
            let mut ranges = highlighter.highlight_line(for_highlighting, syntax_set)?;
            if too_long {
                ranges[0].1 = &line;
            }

            let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], true);
            Ok(escaped)
        } else {
            Ok(line.to_owned())
        }
    }
}

const THEME_FIELD: &str = "dive.theme";
const THEME_DEFAULT: &str = "base16-ocean.dark";
