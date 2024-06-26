#[derive(clap::Parser)]
#[command(about, author, version)]
#[command(allow_missing_positional = true)]
#[command(group = clap::ArgGroup::new("mode").multiple(false).required(true))]
pub(crate) struct Args {
    #[arg(default_value = "HEAD")]
    pub(crate) rev: String,

    #[arg(required = true, group = "mode")]
    pub(crate) file: Option<std::path::PathBuf>,

    #[arg(long, value_name = "PATH", group = "mode")]
    pub(crate) dump_config: Option<std::path::PathBuf>,

    /// Display all supported languages
    #[arg(long, group = "mode")]
    pub(crate) list_languages: bool,

    /// Display all supported highlighting themes
    #[arg(long, group = "mode")]
    pub(crate) list_themes: bool,

    /// Display acknowledgements
    #[arg(long, hide_short_help = true, group = "mode")]
    pub(crate) acknowledgements: bool,

    /// Display information for bug reports.
    #[arg(long, hide_short_help = true, group = "mode")]
    pub(crate) diagnostic: bool,

    /// Run as if git was started in `PATH` instead of the current working directory.
    ///
    /// When multiple `-C` options are given, each subsequent
    /// non-absolute `-C <path>` is interpreted relative to the preceding `-C <path>`. If `<path>` is present but empty, e.g.  `-C ""`, then the
    /// current working directory is left unchanged.
    ///
    /// This option affects options that expect path name like `--git-dir` and `--work-tree` in that their interpretations of the path names
    /// would be made relative to the working directory caused by the `-C` option. For example the following invocations are equivalent:
    ///
    ///     git --git-dir=a.git --work-tree=b -C c status
    ///     git --git-dir=c/a.git --work-tree=c/b status
    #[arg(short = 'C', hide = true, value_name = "PATH")]
    pub(crate) current_dir: Option<Vec<std::path::PathBuf>>,

    #[command(flatten)]
    pub(crate) color: colorchoice_clap::Color,

    #[command(flatten)]
    pub(crate) verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
