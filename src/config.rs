use anyhow::Context as _;

pub struct Config {
    config: Box<dyn ConfigSource>,
}

impl Config {
    pub fn with_repo(repo: &git2::Repository) -> anyhow::Result<Self> {
        let config = repo.config().with_context(|| {
            anyhow::format_err!("failed to read config for {}", repo.path().display())
        })?;
        let config = Box::new(config);
        Ok(Self { config })
    }

    pub fn get<F: Field>(&self, field: &F) -> F::Output {
        field.get_from(&self)
    }
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
        self.config.get_bool(name)
    }
    fn get_i32(&self, name: &str) -> anyhow::Result<i32> {
        self.config.get_i32(name)
    }
    fn get_i64(&self, name: &str) -> anyhow::Result<i64> {
        self.config.get_i64(name)
    }
    fn get_string(&self, name: &str) -> anyhow::Result<String> {
        self.config.get_string(name)
    }
    fn get_path(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.config.get_path(name)
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

pub trait Field {
    type Output;

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

    pub const fn fallback(self, fallback: FallbackFn<R>) -> FallbackField<R> {
        FallbackField {
            field: self,
            fallback,
        }
    }
}

impl<R> Field for RawField<R>
where
    Config: FieldReader<R>,
{
    type Output = Option<R>;

    fn get_from(&self, config: &Config) -> Self::Output {
        config.get_field(self.name).ok()
    }
}

type FallbackFn<R> = fn(&Config) -> R;

pub struct FallbackField<R> {
    field: RawField<R>,
    fallback: FallbackFn<R>,
}

impl<R> Field for FallbackField<R>
where
    Config: FieldReader<R>,
{
    type Output = R;

    fn get_from(&self, config: &Config) -> Self::Output {
        self.field
            .get_from(config)
            .unwrap_or_else(|| (self.fallback)(config))
    }
}
