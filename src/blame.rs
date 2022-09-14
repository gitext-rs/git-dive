use std::io::Write;

use anyhow::Context as _;
use encoding::Encoding as _;
use proc_exit::WithCodeResultExt;

pub fn blame(
    file_path: &std::path::Path,
    config: &mut crate::config::Config,
    args: &crate::args::Args,
    colored_stdout: bool,
    _colored_stderr: bool,
) -> proc_exit::ExitResult {
    let total_width = terminal_size::terminal_size()
        .map(|(w, _h)| w.0)
        .or_else(|| std::env::var_os("COLUMNS").and_then(|s| s.to_str()?.parse::<u16>().ok()))
        .unwrap_or(80) as usize;

    let cwd = std::env::current_dir().with_code(proc_exit::Code::USAGE_ERR)?;
    let repo = git2::Repository::discover(&cwd).with_code(proc_exit::Code::CONFIG_ERR)?;
    config.add_repo(&repo);
    let theme = config.get(&THEME);

    let rev_obj = repo
        .revparse_single(&args.rev)
        .with_code(proc_exit::Code::CONFIG_ERR)?;
    let rev_commit = rev_obj
        .peel_to_commit()
        .map_err(|_| {
            anyhow::format_err!(
                "Unsupported rev `{}` ({})",
                args.rev,
                rev_obj.kind().map(|k| k.str()).unwrap_or("unknown")
            )
        })
        .with_code(proc_exit::Code::CONFIG_ERR)?;
    let mut settings = git2::BlameOptions::new();
    settings
        .track_copies_same_file(true)
        .track_copies_same_commit_moves(true)
        .track_copies_same_commit_copies(true)
        .track_copies_any_commit_copies(true)
        .first_parent(true)
        .ignore_whitespace(true)
        .newest_commit(rev_commit.id());
    let blame = repo
        .blame_file(file_path, Some(&mut settings))
        .with_code(proc_exit::Code::CONFIG_ERR)?;
    let mut annotations = Annotations::new(&repo, &blame);
    annotations
        .relative_origin(&repo, &args.rev)
        .with_code(proc_exit::Code::CONFIG_ERR)?;

    let rel_path = to_repo_relative(file_path, &repo).with_code(proc_exit::Code::CONFIG_ERR)?;
    let file = read_file(&repo, &args.rev, &rel_path).with_code(proc_exit::Code::CONFIG_ERR)?;

    let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let theme_set = syntect::highlighting::ThemeSet::load_defaults();
    let theme = theme_set
        .themes
        .get(&theme)
        .or_else(|| theme_set.themes.get(THEME_DEFAULT))
        .expect("default theme is present");

    let syntax = syntax_set
        .find_syntax_for_file(file_path)?
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let file = convert_file(&file, file_path).with_code(proc_exit::Code::CONFIG_ERR)?;

    let line_count = file.lines().count();
    let line_count_width = line_count.to_string().len(); // bytes = chars = columns with digits
    let sep = "│";
    let space_count = 3;
    let origin_width = annotations
        .notes
        .values()
        .map(|a| {
            // HACK: when we support more than IDs, we'll need to respect UTF-8
            a.origin().len()
        })
        .max()
        .unwrap_or(0);
    let gutter_width = origin_width + line_count_width + sep.len() + space_count;

    let code_width = total_width.saturating_sub(gutter_width);

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
    let gutter_style = anstyle::Style::new()
        .fg_color(
            highlighter
                .theme()
                .settings
                .gutter_foreground
                .map(|c| (c.r, c.g, c.b).into()),
        )
        .bg_color(
            highlighter
                .theme()
                .settings
                .gutter
                .map(|c| (c.r, c.g, c.b).into()),
        );
    let gutter_style = gutter_style.render();
    let wrap = textwrap::Options::new(code_width)
        .break_words(false)
        .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit);

    let mut prev_hunk_id = git2::Oid::zero();
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
            let origin = if i == 0 {
                let hunk = blame.get_line(line_num).unwrap_or_else(|| {
                    panic!("Mismatch in line numbers between dive ({line_num}) and git2")
                });
                let hunk_id = hunk.orig_commit_id();
                if hunk_id != prev_hunk_id {
                    prev_hunk_id = hunk_id;
                    let ann = annotations
                        .notes
                        .get(&hunk_id)
                        .expect("all blame hunks are annotated");
                    ann.origin()
                } else {
                    "⋮"
                }
            } else {
                "⋮"
            };

            let line_num = if i == 0 {
                line_num.to_string()
            } else {
                "⋮".to_owned()
            };
            let _ = write!(
                &mut stdout,
                "{gutter_style}{origin:origin_width$} {line_num:>line_count_width$} {sep} {reset}{visual_line}\n{reset}"
            );
        }
    }

    Ok(())
}

fn to_repo_relative(
    path: &std::path::Path,
    repo: &git2::Repository,
) -> anyhow::Result<std::path::PathBuf> {
    let workdir = repo.workdir().ok_or_else(|| {
        anyhow::format_err!("No workdir found; Bare repositories are not supported")
    })?;
    let abs_path = path
        .canonicalize()
        .with_context(|| anyhow::format_err!("Could not read {}", path.display()))?;
    let rel_path = abs_path.strip_prefix(&workdir).map_err(|_| {
        anyhow::format_err!(
            "File {} is not in the repository's workdir {}",
            abs_path.display(),
            workdir.display()
        )
    })?;
    Ok(rel_path.to_owned())
}

fn read_file<'r>(
    repo: &'r git2::Repository,
    rev: &str,
    rel_path: &std::path::Path,
) -> anyhow::Result<Vec<u8>> {
    let rev_obj = repo.revparse_single(rev)?;
    let rev_tree = rev_obj.peel_to_tree().map_err(|_| {
        anyhow::format_err!(
            "Unsupported rev `{}` ({})",
            rev,
            rev_obj.kind().map(|k| k.str()).unwrap_or("unknown")
        )
    })?;
    let file_entry = rev_tree
        .get_path(rel_path)
        .with_context(|| format!("Could not read {} at {}", rel_path.display(), rev))?;
    let file_obj = file_entry
        .to_object(repo)
        .with_context(|| format!("Could not read {} at {}", rel_path.display(), rev))?;
    let file_blob = file_obj
        .as_blob()
        .with_context(|| format!("Could not read {} at {}", rel_path.display(), rev))?;
    let file = file_blob.content();
    Ok(file.to_owned())
}

fn convert_file(buffer: &[u8], path: &std::path::Path) -> anyhow::Result<String> {
    let content_type = content_inspector::inspect(buffer);

    let buffer = match content_type {
        content_inspector::ContentType::BINARY |
        // HACK: We don't support UTF-32 yet
        content_inspector::ContentType::UTF_32LE |
        content_inspector::ContentType::UTF_32BE => {
            anyhow::bail!("Could not ready binary file {}", path.display())
        },
        content_inspector::ContentType::UTF_8 |
        content_inspector::ContentType::UTF_8_BOM => {
            String::from_utf8_lossy(buffer).into_owned()
        },
        content_inspector::ContentType::UTF_16LE => {
            let buffer = encoding::all::UTF_16LE.decode(buffer, encoding::DecoderTrap::Replace)
                .map_err(|_| anyhow::format_err!("Could not decode UTF-16 in {}", path.display()))?;
            buffer
        }
        content_inspector::ContentType::UTF_16BE => {
            let buffer = encoding::all::UTF_16BE.decode(buffer, encoding::DecoderTrap::Replace)
                .map_err(|_| anyhow::format_err!("Could not decode UTF-16 in {}", path.display()))?;
            buffer
        },
    };

    Ok(buffer)
}

pub struct Annotations {
    notes: std::collections::HashMap<git2::Oid, Annotation>,
}

impl Annotations {
    pub fn new(repo: &git2::Repository, blame: &git2::Blame<'_>) -> Self {
        let mut notes = std::collections::HashMap::new();
        for hunk in blame.iter() {
            let id = hunk.orig_commit_id();
            notes.entry(id).or_insert_with(|| Annotation::new(repo, id));
        }

        Annotations { notes }
    }

    pub fn relative_origin(&mut self, repo: &git2::Repository, head: &str) -> anyhow::Result<()> {
        let mut queue = self
            .notes
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>();

        let rev_obj = repo.revparse_single(head)?;
        let rev_commit = rev_obj.peel_to_commit().map_err(|_| {
            anyhow::format_err!(
                "Unsupported rev `{}` ({})",
                head,
                rev_obj.kind().map(|k| k.str()).unwrap_or("unknown")
            )
        })?;

        let mut revwalk = repo.revwalk()?;
        revwalk.simplify_first_parent()?;
        revwalk.push(rev_commit.id())?;
        for (i, id) in revwalk.enumerate() {
            let id = id?;
            let relative = if i == 0 {
                head.to_owned()
            } else {
                format!("{head}~{i}")
            };
            self.notes
                .entry(id)
                .or_insert_with(|| Annotation::new(repo, id))
                .relative = Some(relative);

            queue.remove(&id);
            if queue.is_empty() {
                break;
            }
        }
        Ok(())
    }
}

pub struct Annotation {
    short: String,
    relative: Option<String>,
}

impl Annotation {
    pub fn new(repo: &git2::Repository, id: git2::Oid) -> Self {
        let obj = repo.find_object(id, None).expect("blame has valid ids");
        let short = obj
            .short_id()
            .unwrap_or_else(|e| panic!("unknown failure for short_id for {}: {}", id, e))
            .as_str()
            .expect("short_id is always valid UTF-8")
            .to_owned();
        Self {
            short,
            relative: None,
        }
    }

    pub fn origin(&self) -> &str {
        self.relative.as_deref().unwrap_or(self.short.as_str())
    }
}

pub struct Highlighter<'a> {
    highlighter: Option<syntect::easy::HighlightLines<'a>>,
    theme: &'a syntect::highlighting::Theme,
}

impl<'a> Highlighter<'a> {
    fn enabled(
        syntax: &'a syntect::parsing::SyntaxReference,
        theme: &'a syntect::highlighting::Theme,
    ) -> Self {
        let highlighter = Some(syntect::easy::HighlightLines::new(syntax, theme));
        Self { highlighter, theme }
    }

    fn disabled() -> Self {
        let highlighter = None;
        static THEME: syntect::highlighting::Theme = syntect::highlighting::Theme {
            name: None,
            author: None,
            settings: syntect::highlighting::ThemeSettings {
                foreground: None,
                background: None,
                caret: None,
                line_highlight: None,
                misspelling: None,
                minimap_border: None,
                accent: None,
                popup_css: None,
                phantom_css: None,
                bracket_contents_foreground: None,
                bracket_contents_options: None,
                brackets_foreground: None,
                brackets_background: None,
                brackets_options: None,
                tags_foreground: None,
                tags_options: None,
                highlight: None,
                find_highlight: None,
                find_highlight_foreground: None,
                gutter: None,
                gutter_foreground: None,
                selection: None,
                selection_foreground: None,
                selection_border: None,
                inactive_selection: None,
                inactive_selection_foreground: None,
                guide: None,
                active_guide: None,
                stack_guide: None,
                shadow: None,
            },
            scopes: Vec::new(),
        };
        Self {
            highlighter,
            theme: &THEME,
        }
    }

    fn theme(&self) -> &syntect::highlighting::Theme {
        self.theme
    }

    fn highlight_line(
        &mut self,
        line: &str,
        syntax_set: &syntect::parsing::SyntaxSet,
    ) -> anyhow::Result<String> {
        if let Some(highlighter) = &mut self.highlighter {
            // skip syntax highlighting on long lines
            let too_long = line.len() > 1024 * 16;
            let for_highlighting: &str = if too_long { "\n" } else { line };
            let mut ranges = highlighter.highlight_line(for_highlighting, syntax_set)?;
            if too_long {
                ranges[0].1 = line;
            }

            let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], true);
            Ok(escaped)
        } else {
            Ok(line.to_owned())
        }
    }
}

const THEME_DEFAULT: &str = "base16-ocean.dark";
pub const THEME: crate::config::FallbackField<String> =
    crate::config::RawField::<String>::new("dive.theme").fallback(|_| THEME_DEFAULT.to_owned());
