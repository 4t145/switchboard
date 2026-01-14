use std::{ops::Deref, str::FromStr};

use crate::Resolver;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct StringParseResolver<R>(R);

impl<R> StringParseResolver<R> {
    pub fn new(resolver: R) -> Self {
        Self(resolver)
    }
    pub fn into_inner(self) -> R {
        self.0
    }
}

impl<R> Deref for StringParseResolver<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StringParseResolveError<ParseError, ResolveError> {
    #[error("Parse error: {source}")]
    ParseError {
        #[source]
        source: ParseError,
    },
    #[error("resolve error: {source}")]
    ResolveError {
        #[source]
        source: ResolveError,
    },
}
impl<L, T, R> Resolver<L, T> for StringParseResolver<R>
where
    L: Send + Sync + 'static,
    T: FromStr + Send + Sync + 'static,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    R: Resolver<L, String> + Send + Sync + 'static,
{
    type Error = StringParseResolveError<T::Err, R::Error>;
    async fn resolve(&self, link: L) -> Result<T, Self::Error> {
        let string = self
            .0
            .resolve(link)
            .await
            .map_err(|e| StringParseResolveError::ResolveError { source: e })?;
        let value = string
            .parse::<T>()
            .map_err(|e| StringParseResolveError::ParseError { source: e })?;
        Ok(value)
    }
}
