use once_cell::sync::Lazy;
use std::str::FromStr;

pub(crate) struct GlobalContext {}

fn initialize_as_or_default<T>(env_var: &str, default: T) -> T
where
    T: From<String>,
{
    std::env::var(env_var).ok().map(T::from).unwrap_or(default)
}

fn try_initialize_as_or_default<T>(env_var: &str, default: T) -> T
where
    T: FromStr,
{
    std::env::var(env_var)
        .ok()
        .as_ref()
        .and_then(|e| T::from_str(e).ok())
        .unwrap_or(default)
}

pub(crate) static GLOBAL_CONTEXT: Lazy<GlobalContext> = Lazy::new(|| GlobalContext {});
