use anyhow::Context as _;

pub trait ConfigExt {
    fn get<F: Field>(&self, field: &F) -> F::Output;
}

impl ConfigExt for git2::Config {
    fn get<F: Field>(&self, field: &F) -> F::Output {
        field.get_from(self)
    }
}

pub trait FieldReader<T> {
    fn get_field(&self, name: &str) -> anyhow::Result<T>;
}

impl FieldReader<bool> for git2::Config {
    fn get_field(&self, name: &str) -> anyhow::Result<bool> {
        self.get_bool(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl FieldReader<i32> for git2::Config {
    fn get_field(&self, name: &str) -> anyhow::Result<i32> {
        self.get_i32(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl FieldReader<i64> for git2::Config {
    fn get_field(&self, name: &str) -> anyhow::Result<i64> {
        self.get_i64(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl FieldReader<String> for git2::Config {
    fn get_field(&self, name: &str) -> anyhow::Result<String> {
        self.get_string(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

impl FieldReader<std::path::PathBuf> for git2::Config {
    fn get_field(&self, name: &str) -> anyhow::Result<std::path::PathBuf> {
        self.get_path(name)
            .with_context(|| anyhow::format_err!("failed to read `{}`", name))
    }
}

pub trait Field {
    type Output;

    fn get_from(&self, config: &git2::Config) -> Self::Output;
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
    git2::Config: FieldReader<R>,
{
    type Output = Option<R>;

    fn get_from(&self, config: &git2::Config) -> Self::Output {
        config.get_field(self.name).ok()
    }
}

type FallbackFn<R> = fn(&git2::Config) -> R;

pub struct FallbackField<R> {
    field: RawField<R>,
    fallback: FallbackFn<R>,
}

impl<R> Field for FallbackField<R>
where
    git2::Config: FieldReader<R>,
{
    type Output = R;

    fn get_from(&self, config: &git2::Config) -> Self::Output {
        self.field
            .get_from(config)
            .unwrap_or_else(|| (self.fallback)(config))
    }
}
