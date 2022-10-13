#![allow(clippy::collapsible_else_if)]
#![allow(clippy::let_and_return)]
#![allow(clippy::if_same_then_else)]

use clap::Parser;
use proc_exit::prelude::*;

mod args;
mod assets;
mod blame;
mod config;
mod git2_config;
mod git_pager;
mod logger;

use crate::git2_config::Config;
use crate::git_pager::Pager;

fn main() {
    human_panic::setup_panic!();
    let result = run();
    proc_exit::exit(result);
}

fn run() -> proc_exit::ExitResult {
    let mut config = Config::system();
    concolor::set(match config.get(&crate::git2_config::COLOR_UI) {
        crate::git2_config::ColorWhen::Always => concolor::ColorChoice::Always,
        crate::git2_config::ColorWhen::Auto => concolor::ColorChoice::Auto,
        crate::git2_config::ColorWhen::Never => concolor::ColorChoice::Never,
    });

    // clap's `get_matches` uses Failure rather than Usage, so bypass it for `get_matches_safe`.
    let args = args::Args::parse();

    args.color.apply();
    let colored_stdout = concolor::get(concolor::Stream::Stdout).ansi_color();
    let colored_stderr = concolor::get(concolor::Stream::Stderr).ansi_color();

    logger::init_logging(args.verbose.clone(), colored_stderr);

    if let Some(current_dir) = args.current_dir.as_deref() {
        let current_dir = current_dir
            .iter()
            .fold(std::path::PathBuf::new(), |current, next| {
                current.join(next)
            });
        log::trace!("CWD={}", current_dir.display());
        std::env::set_current_dir(current_dir).with_code(proc_exit::Code::FAILURE)?;
    }

    if let Some(output_path) = args.dump_config.as_deref() {
        config::dump_config(output_path, &mut config)?;
    } else if args.list_languages {
        list_languages(&mut config, colored_stdout)?;
    } else if args.list_themes {
        list_themes(&mut config, colored_stdout)?;
    } else if let Some(file_path) = args.file.as_deref() {
        blame::blame(
            file_path,
            &mut config,
            &args,
            colored_stdout,
            colored_stderr,
        )?;
    } else {
        unreachable!("clap ensured a mode exists");
    }

    Ok(())
}

fn list_languages(config: &mut Config, colored_stdout: bool) -> proc_exit::ExitResult {
    let total_width = terminal_size::terminal_size()
        .map(|(w, _h)| w.0)
        .or_else(|| std::env::var_os("COLUMNS").and_then(|s| s.to_str()?.parse::<u16>().ok()))
        .unwrap_or(80) as usize;

    let pager = config.get(&crate::git2_config::PAGER);
    let mut pager = Pager::stdout(&pager);
    let mut pager = pager.start();
    let pager = pager.as_writer().with_code(proc_exit::Code::FAILURE)?;

    let syntax_set = assets::load_syntaxes();
    let name_width = syntax_set
        .syntaxes()
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0)
        + 1;
    let syntax_width = total_width - name_width;
    let wrap = textwrap::Options::new(syntax_width)
        .break_words(false)
        .word_separator(textwrap::WordSeparator::AsciiSpace)
        .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit);
    for syntax in syntax_set.syntaxes() {
        let ext = syntax.file_extensions.join(", ");
        let ext = textwrap::wrap(&ext, &wrap);
        for (i, ext_line) in ext.into_iter().enumerate() {
            let mut name = if i == 0 {
                syntax.name.clone()
            } else {
                "".to_owned()
            };
            let mut ext_line = ext_line.into_owned();
            if colored_stdout {
                name = format!(
                    "{}{}{}",
                    anstyle::Effects::BOLD.render(),
                    name,
                    anstyle::Reset.render()
                );
                ext_line = format!(
                    "{}{}{}",
                    anstyle::AnsiColor::Green.render_fg(),
                    ext_line,
                    anstyle::Reset.render()
                );
            }
            let _ = writeln!(pager, "{:<width$}{}", name, ext_line, width = name_width);
        }
    }

    Ok(())
}

fn list_themes(config: &mut Config, colored_stdout: bool) -> proc_exit::ExitResult {
    let pager = config.get(&crate::git2_config::PAGER);
    let mut pager = Pager::stdout(&pager);
    let mut pager = pager.start();
    let pager = pager.as_writer().with_code(proc_exit::Code::FAILURE)?;

    let theme_set = assets::load_themes();
    if colored_stdout {
        let syntax_set = assets::load_syntaxes();
        let syntax = syntax_set
            .find_syntax_by_name("Rust")
            .expect("always included");
        for (name, theme) in theme_set.themes.iter() {
            let mut highlighter = blame::Highlighter::enabled(syntax, theme);
            let _ = writeln!(
                pager,
                "Theme: {}{}{}",
                anstyle::Effects::BOLD.render(),
                name,
                anstyle::Reset.render()
            );
            let _ = writeln!(pager);
            for line in THEME_PREVIEW_DATA.lines() {
                let _ = writeln!(
                    pager,
                    "{}{}",
                    highlighter.highlight_line(line, &syntax_set).unwrap(),
                    anstyle::Reset.render()
                );
            }
            let _ = writeln!(pager);
        }
    } else {
        for name in theme_set.themes.keys() {
            let _ = writeln!(pager, "{}", name);
        }
    }
    Ok(())
}

const THEME_PREVIEW_DATA: &str = include_str!("../assets/theme_preview.rs");
