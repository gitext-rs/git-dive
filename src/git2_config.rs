use anyhow::Context as _;

#[derive(Debug)]
pub(crate) struct Config {
    pager: InMemoryConfig,
    system: Option<GitConfig>,
    xdg: Option<GitConfig>,
    global: Option<GitConfig>,
    local: Option<GitConfig>,
    env: InMemoryConfig,
    cli: InMemoryConfig,
    git_pager: InMemoryConfig,
}

impl Config {
    pub(crate) fn system() -> Self {
        let pager = InMemoryConfig::pager();
        let system = GitConfig::open_system();
        let xdg = GitConfig::open_xdg();
        let global = GitConfig::open_global();
        let local = None;
        let env = InMemoryConfig::git_env();
        let cli = InMemoryConfig::git_cli();
        let git_pager = InMemoryConfig::git_pager();
        Self {
            pager,
            system,
            xdg,
            global,
            local,
            env,
            cli,
            git_pager,
        }
    }

    pub(crate) fn add_repo(&mut self, repo: &git2::Repository) {
        let local = GitConfig::open_local(repo);
        self.local = local;
    }

    pub(crate) fn get<F: Field>(&self, field: &F) -> F::Output {
        field.get_from(self)
    }

    pub(crate) fn dump<'f>(
        &self,
        fields: impl IntoIterator<Item = &'f dyn ReflectField>,
    ) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        let mut prior_section = "";
        for field in fields {
            let (section, name) = field
                .name()
                .split_once('.')
                .unwrap_or_else(|| panic!("field `{}` is missing a section", field.name()));
            if section != prior_section {
                let _ = writeln!(&mut output, "[{section}]");
                prior_section = section;
            }
            let value = field.dump(self);
            let source = field.get_source(self);
            let _ = writeln!(&mut output, "\t{name} = {value}  # {source}");
        }

        output
    }

    pub(crate) fn sources(&self) -> impl Iterator<Item = &dyn ConfigSource> {
        [
            Some(&self.git_pager).map(|c| c as &dyn ConfigSource),
            Some(&self.cli).map(|c| c as &dyn ConfigSource),
            Some(&self.env).map(|c| c as &dyn ConfigSource),
            self.local.as_ref().map(|c| c as &dyn ConfigSource),
            self.global.as_ref().map(|c| c as &dyn ConfigSource),
            self.xdg.as_ref().map(|c| c as &dyn ConfigSource),
            self.system.as_ref().map(|c| c as &dyn ConfigSource),
            Some(&self.pager).map(|c| c as &dyn ConfigSource),
        ]
        .into_iter()
        .flatten()
    }
}

pub(crate) trait ConfigSource {
    fn name(&self) -> &str;

    fn get_source(&self, name: &str) -> anyhow::Result<&str>;
    fn get_bool(&self, name: &str) -> anyhow::Result<bool>;
    fn get_i32(&self, name: &str) -> anyhow::Result<i32>;
    fn get_i64(&self, name: &str) -> anyhow::Result<i64>;
    fn get_string(&self, name: &str) -> anyhow::Result<String>;
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf>;
}

impl ConfigSource for Config {
    fn name(&self) -> &str {
        "git"
    }

    fn get_source(&self, name: &str) -> anyhow::Result<&str> {
        for config in self.sources() {
            if let Ok(source) = config.get_source(name) {
                return Ok(source);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_source(name)
    }

    fn get_bool(&self, name: &str) -> anyhow::Result<bool> {
        for config in self.sources() {
            if let Ok(v) = config.get_bool(name) {
                return Ok(v);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_bool(name)
    }
    fn get_i32(&self, name: &str) -> anyhow::Result<i32> {
        for config in self.sources() {
            if let Ok(v) = config.get_i32(name) {
                return Ok(v);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_i32(name)
    }
    fn get_i64(&self, name: &str) -> anyhow::Result<i64> {
        for config in self.sources() {
            if let Ok(v) = config.get_i64(name) {
                return Ok(v);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_i64(name)
    }
    fn get_string(&self, name: &str) -> anyhow::Result<String> {
        for config in self.sources() {
            if let Ok(v) = config.get_string(name) {
                return Ok(v);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_string(name)
    }
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        for config in self.sources() {
            if let Ok(v) = config.get_path(name) {
                return Ok(v);
            }
        }
        // Fallback to the first error
        self.sources()
            .next()
            .expect("always a source")
            .get_path(name)
    }
}

impl ConfigSource for git2::Config {
    fn name(&self) -> &str {
        "gitconfig"
    }

    fn get_source(&self, name: &str) -> anyhow::Result<&str> {
        self.get_entry(name)
            .map(|_| self.name())
            .map_err(|e| e.into())
    }
    fn get_bool(&self, name: &str) -> anyhow::Result<bool> {
        self.get_bool(name).map_err(|e| e.into())
    }
    fn get_i32(&self, name: &str) -> anyhow::Result<i32> {
        self.get_i32(name).map_err(|e| e.into())
    }
    fn get_i64(&self, name: &str) -> anyhow::Result<i64> {
        self.get_i64(name).map_err(|e| e.into())
    }
    fn get_string(&self, name: &str) -> anyhow::Result<String> {
        self.get_string(name).map_err(|e| e.into())
    }
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.get_path(name).map_err(|e| e.into())
    }
}

pub(crate) struct GitConfig {
    name: String,
    config: git2::Config,
}

impl GitConfig {
    pub(crate) fn open_system() -> Option<Self> {
        let path = git2::Config::find_system().ok()?;
        Self::open_path(&path)
    }

    pub(crate) fn open_xdg() -> Option<Self> {
        let path = git2::Config::find_xdg().ok()?;
        Self::open_path(&path)
    }

    pub(crate) fn open_global() -> Option<Self> {
        let path = git2::Config::find_global().ok()?;
        Self::open_path(&path)
    }

    pub(crate) fn open_local(repo: &git2::Repository) -> Option<Self> {
        let path = repo.path().join("config");
        let config = git2::Config::open(&path).ok()?;
        let name = "$GIT_DIR/config".to_owned();
        Some(Self { name, config })
    }

    fn open_path(path: &std::path::Path) -> Option<Self> {
        let config = git2::Config::open(path).ok()?;
        let name = path.display().to_string();
        Some(Self { name, config })
    }

    fn inner(&self) -> &impl ConfigSource {
        &self.config
    }
}

impl ConfigSource for GitConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_source(&self, name: &str) -> anyhow::Result<&str> {
        self.inner().get_source(name)
    }
    fn get_bool(&self, name: &str) -> anyhow::Result<bool> {
        self.inner().get_bool(name)
    }
    fn get_i32(&self, name: &str) -> anyhow::Result<i32> {
        self.inner().get_i32(name)
    }
    fn get_i64(&self, name: &str) -> anyhow::Result<i64> {
        self.inner().get_i64(name)
    }
    fn get_string(&self, name: &str) -> anyhow::Result<String> {
        self.inner().get_string(name)
    }
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.inner().get_path(name)
    }
}

impl std::fmt::Debug for GitConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitConfig")
            .field("name", &self.name)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct InMemoryConfig {
    name: String,
    values: std::collections::BTreeMap<String, Vec<String>>,
}

impl InMemoryConfig {
    pub(crate) fn git_env() -> Self {
        Self::from_env("git-config-env", git_config_env::ConfigEnv::new().iter())
    }

    pub(crate) fn git_cli() -> Self {
        Self::from_env(
            "git-cli",
            git_config_env::ConfigParameters::new()
                .iter()
                .map(|(k, v)| (k, v.unwrap_or(std::borrow::Cow::Borrowed("true")))),
        )
    }

    pub(crate) fn git_pager() -> Self {
        let name = "GIT_PAGER";
        let value = std::env::var_os(name).and_then(|v| v.into_string().ok());
        Self::from_env(name, value.map(|v| ("core.pager", v)))
    }

    pub(crate) fn pager() -> Self {
        let name = "PAGER";
        let value = std::env::var_os(name).and_then(|v| v.into_string().ok());
        Self::from_env(name, value.map(|v| ("core.pager", v)))
    }

    pub(crate) fn from_env(
        name: impl Into<String>,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let name = name.into();
        let mut values = std::collections::BTreeMap::new();
        for (key, value) in env {
            values
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(value.into());
        }
        Self { name, values }
    }

    fn get_str(&self, name: &str) -> anyhow::Result<&str> {
        let value = self
            .values
            .get(name)
            .context("field is missing")?
            .last()
            .expect("always at least one element");
        Ok(value)
    }
}

impl Default for InMemoryConfig {
    fn default() -> Self {
        Self {
            name: "null".to_owned(),
            values: Default::default(),
        }
    }
}

impl ConfigSource for InMemoryConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_source(&self, name: &str) -> anyhow::Result<&str> {
        self.get_str(name).map(|_| self.name())
    }
    fn get_bool(&self, name: &str) -> anyhow::Result<bool> {
        let v = self.get_str(name).unwrap_or("true");
        v.parse::<bool>().map_err(|e| e.into())
    }
    fn get_i32(&self, name: &str) -> anyhow::Result<i32> {
        self.get_str(name)
            .and_then(|v| v.parse::<i32>().map_err(|e| e.into()))
    }
    fn get_i64(&self, name: &str) -> anyhow::Result<i64> {
        self.get_str(name)
            .and_then(|v| v.parse::<i64>().map_err(|e| e.into()))
    }
    fn get_string(&self, name: &str) -> anyhow::Result<String> {
        self.get_str(name).map(|v| v.to_owned())
    }
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.get_string(name).map(|v| v.into())
    }
}

pub(crate) trait FieldReader<T> {
    fn get_field(&self, name: &str) -> anyhow::Result<T>;
}

impl<C: ConfigSource> FieldReader<bool> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<bool> {
        self.get_bool(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
    }
}

impl<C: ConfigSource> FieldReader<i32> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<i32> {
        self.get_i32(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
    }
}

impl<C: ConfigSource> FieldReader<i64> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<i64> {
        self.get_i64(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
    }
}

impl<C: ConfigSource> FieldReader<String> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<String> {
        self.get_string(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
    }
}

impl<C: ConfigSource> FieldReader<std::path::PathBuf> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.get_path(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
    }
}

impl<P: Parseable, C: ConfigSource> FieldReader<P> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<P> {
        self.get_string(name)
            .with_context(|| anyhow::format_err!("failed to read `{name}`"))
            .and_then(|s| P::parse(&s))
    }
}

pub(crate) trait Parseable: Sized {
    fn parse(s: &str) -> anyhow::Result<Self>;
}

pub(crate) trait Field {
    type Output;

    fn name(&self) -> &'static str;
    fn get_from(&self, config: &Config) -> Self::Output;
    fn get_source<'c>(&self, config: &'c Config) -> Option<&'c str>;
}

pub(crate) struct RawField<R> {
    name: &'static str,
    _type: std::marker::PhantomData<R>,
}

impl<R> RawField<R> {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            name,
            _type: std::marker::PhantomData,
        }
    }

    pub(crate) const fn default_value(self, default: DefaultFn<R>) -> DefaultField<R> {
        DefaultField {
            field: self,
            default,
        }
    }
}

impl<R> RawField<R>
where
    R: Default,
{
    pub(crate) const fn default(self) -> DefaultField<R> {
        DefaultField {
            field: self,
            default: R::default,
        }
    }
}

impl<R> Field for RawField<R>
where
    Config: FieldReader<R>,
{
    type Output = Option<R>;

    fn name(&self) -> &'static str {
        self.name
    }

    fn get_from(&self, config: &Config) -> Self::Output {
        config.get_field(self.name).ok()
    }

    fn get_source<'c>(&self, config: &'c Config) -> Option<&'c str> {
        config.get_source(self.name).ok()
    }
}

type DefaultFn<R> = fn() -> R;

pub(crate) struct DefaultField<R> {
    field: RawField<R>,
    default: DefaultFn<R>,
}

impl<R> Field for DefaultField<R>
where
    Config: FieldReader<R>,
{
    type Output = R;

    fn name(&self) -> &'static str {
        self.field.name()
    }

    fn get_from(&self, config: &Config) -> Self::Output {
        self.field
            .get_from(config)
            .unwrap_or_else(|| (self.default)())
    }

    fn get_source<'c>(&self, config: &'c Config) -> Option<&'c str> {
        Some(self.field.get_source(config).unwrap_or("default"))
    }
}

pub(crate) trait ReflectField {
    fn name(&self) -> &'static str;

    fn dump(&self, config: &Config) -> String;
    fn get_source<'c>(&self, config: &'c Config) -> &'c str;
}

impl<F> ReflectField for F
where
    F: Field,
    F::Output: std::fmt::Display,
{
    fn name(&self) -> &'static str {
        self.name()
    }

    fn dump(&self, config: &Config) -> String {
        self.get_from(config).to_string()
    }
    fn get_source<'c>(&self, config: &'c Config) -> &'c str {
        F::get_source(self, config).expect("assuming if its Display then it has a source")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) enum ColorWhen {
    Always,
    #[default]
    Auto,
    Never,
}

impl ColorWhen {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Auto => "auto",
            Self::Never => "never",
        }
    }
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::str::FromStr for ColorWhen {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" | "true" => Ok(Self::Always),
            "auto" => Ok(Self::Auto),
            "never" | "false" => Ok(Self::Never),
            _ => Err(anyhow::format_err!("unsupported color valued: `{s}`")),
        }
    }
}

impl Parseable for ColorWhen {
    fn parse(s: &str) -> anyhow::Result<Self> {
        <Self as std::str::FromStr>::from_str(s)
    }
}

pub(crate) const COLOR_UI: DefaultField<ColorWhen> =
    RawField::<ColorWhen>::new("color.ui").default();

pub(crate) const PAGER: DefaultField<String> =
    RawField::<String>::new("core.pager").default_value(|| "less".to_owned());
