use anyhow::Context as _;

pub struct Config {
    system: Option<git2::Config>,
    repo: Option<git2::Config>,
    env: InMemoryConfig,
    cli: InMemoryConfig,
}

impl Config {
    pub fn system() -> Self {
        let system = git2::Config::open_default().ok();
        let repo = None;
        let env = InMemoryConfig::git_env();
        let cli = InMemoryConfig::git_cli();
        Self {
            system,
            repo,
            env,
            cli,
        }
    }

    pub fn add_repo(&mut self, repo: &git2::Repository) {
        let config_path = git_dir_config(repo);
        let repo = git2::Config::open(&config_path).ok();
        self.repo = repo;
    }

    pub fn get<F: Field>(&self, field: &F) -> F::Output {
        field.get_from(self)
    }

    pub fn dump<'f>(&self, fields: impl IntoIterator<Item = &'f dyn ReflectField>) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        let mut prior_section = "";
        for field in fields {
            let (section, name) = field
                .name()
                .split_once('.')
                .unwrap_or_else(|| panic!("field `{}` is missing a section", field.name()));
            if section != prior_section {
                let _ = writeln!(&mut output, "[{}]", section);
                prior_section = section;
            }
            let _ = writeln!(&mut output, "\t{} = {}", name, field.dump(self));
        }

        output
    }

    pub fn sources(&self) -> impl Iterator<Item = &dyn ConfigSource> {
        [
            Some(&self.cli).map(|c| c as &dyn ConfigSource),
            Some(&self.env).map(|c| c as &dyn ConfigSource),
            self.repo.as_ref().map(|c| c as &dyn ConfigSource),
            self.system.as_ref().map(|c| c as &dyn ConfigSource),
        ]
        .into_iter()
        .flatten()
    }
}

fn git_dir_config(repo: &git2::Repository) -> std::path::PathBuf {
    repo.path().join("config")
}

pub trait ConfigSource {
    fn name(&self) -> &str;

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
        "git"
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

pub struct InMemoryConfig {
    name: String,
    values: std::collections::BTreeMap<String, Vec<String>>,
}

impl InMemoryConfig {
    pub fn git_env() -> Self {
        Self::from_env("git-config-env", git_config_env::ConfigEnv::new().iter())
    }

    pub fn git_cli() -> Self {
        Self::from_env(
            "git-cli",
            git_config_env::ConfigParameters::new()
                .iter()
                .map(|(k, v)| (k, v.unwrap_or(std::borrow::Cow::Borrowed("true")))),
        )
    }

    pub fn from_env(
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

pub trait FieldReader<T> {
    fn get_field(&self, name: &str) -> anyhow::Result<T>;
}

impl<C: ConfigSource> FieldReader<bool> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<bool> {
        self.get_bool(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl<C: ConfigSource> FieldReader<i32> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<i32> {
        self.get_i32(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl<C: ConfigSource> FieldReader<i64> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<i64> {
        self.get_i64(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl<C: ConfigSource> FieldReader<String> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<String> {
        self.get_string(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl<C: ConfigSource> FieldReader<std::path::PathBuf> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.get_path(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl<P: Parseable, C: ConfigSource> FieldReader<P> for C {
    fn get_field(&self, name: &str) -> anyhow::Result<P> {
        self.get_string(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
            .and_then(|s| P::parse(&s))
    }
}

pub trait Parseable: Sized {
    fn parse(s: &str) -> anyhow::Result<Self>;
}

pub struct ParseWrapper<T>(pub T);

impl<T: std::fmt::Display> std::fmt::Display for ParseWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> std::str::FromStr for ParseWrapper<T>
where
    T: std::str::FromStr,
    T::Err: Into<anyhow::Error>,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(ParseWrapper).map_err(|e| e.into())
    }
}

impl<T> Parseable for ParseWrapper<T>
where
    T: Parseable,
    T: std::str::FromStr,
    T: std::fmt::Display,
    T::Err: Into<anyhow::Error>,
{
    fn parse(s: &str) -> anyhow::Result<Self> {
        <Self as std::str::FromStr>::from_str(s)
    }
}

pub trait Field {
    type Output;

    fn name(&self) -> &'static str;
    fn get_from(&self, config: &Config) -> Self::Output;
}

pub struct RawField<R> {
    name: &'static str,
    _type: std::marker::PhantomData<R>,
}

impl<R> RawField<R> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _type: std::marker::PhantomData,
        }
    }

    pub const fn default_value(self, default: DefaultFn<R>) -> DefaultField<R> {
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
    pub const fn default(self) -> DefaultField<R> {
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
}

type DefaultFn<R> = fn() -> R;

pub struct DefaultField<R> {
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
}

pub trait ReflectField {
    fn name(&self) -> &'static str;

    fn dump(&self, config: &Config) -> String;
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
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl ColorWhen {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Auto => "auto",
            Self::Never => "never",
        }
    }
}

impl Default for ColorWhen {
    fn default() -> Self {
        Self::Auto
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
            _ => Err(anyhow::format_err!("unsupported color valued: `{}`", s)),
        }
    }
}

impl Parseable for ColorWhen {
    fn parse(s: &str) -> anyhow::Result<Self> {
        <Self as std::str::FromStr>::from_str(s)
    }
}

pub const COLOR_UI: DefaultField<ColorWhen> = RawField::<ColorWhen>::new("color.ui").default();
