#[derive(clap::Parser)]
#[clap(about, author, version)]
#[clap(
        setting = clap::AppSettings::DeriveDisplayOrder,
        dont_collapse_args_in_usage = true,
        color = concolor_clap::color_choice(),
        allow_missing_positional = true,
    )]
#[clap(group = clap::ArgGroup::new("mode").multiple(false).required(true))]
pub struct Args {
    #[clap(action, default_value = "HEAD")]
    pub rev: String,

    #[clap(required = true, action, group = "mode")]
    pub file: Option<std::path::PathBuf>,

    #[clap(long, value_name = "PATH", action, group = "mode")]
    pub dump_config: Option<std::path::PathBuf>,

    /// Run as if git was started in `PATH` instead of the current working directory.
    ///
    /// When multiple -C options are given, each subsequent
    /// non-absolute -C <path> is interpreted relative to the preceding -C <path>. If <path> is present but empty, e.g.  -C "", then the
    /// current working directory is left unchanged.
    ///
    /// This option affects options that expect path name like --git-dir and --work-tree in that their interpretations of the path names
    /// would be made relative to the working directory caused by the -C option. For example the following invocations are equivalent:
    ///
    ///     git --git-dir=a.git --work-tree=b -C c status
    ///     git --git-dir=c/a.git --work-tree=c/b status
    #[clap(short = 'C', hide = true, value_name = "PATH", parse(from_os_str))]
    pub current_dir: Option<Vec<std::path::PathBuf>>,

    #[clap(flatten)]
    pub(crate) color: concolor_clap::Color,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }
}
