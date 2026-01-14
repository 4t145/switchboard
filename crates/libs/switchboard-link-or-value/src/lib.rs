pub mod resolver;

#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    Clone,
    PartialEq,
    Eq,
    Hash,
)]
#[serde(untagged)]
pub enum LinkOrValue<L, V> {
    Link(L),
    Value(V),
}

impl<L, V> LinkOrValue<L, V> {
    pub fn is_link(&self) -> bool {
        matches!(self, LinkOrValue::Link(_))
    }
    pub fn as_link(&self) -> Option<&L> {
        match self {
            LinkOrValue::Link(link) => Some(link),
            _ => None,
        }
    }
    pub fn as_value(&self) -> Option<&V> {
        match self {
            LinkOrValue::Value(value) => Some(value),
            _ => None,
        }
    }
    pub fn into_value(self) -> Option<V> {
        match self {
            LinkOrValue::Value(value) => Some(value),
            _ => None,
        }
    }
    pub fn into_link(self) -> Option<L> {
        match self {
            LinkOrValue::Link(link) => Some(link),
            _ => None,
        }
    }
    pub fn unwrap_value(self) -> V {
        match self {
            LinkOrValue::Value(value) => value,
            _ => panic!("Called unwrap_value on a Link variant"),
        }
    }
    pub fn unwrap_link(self) -> L {
        match self {
            LinkOrValue::Link(link) => link,
            _ => panic!("Called unwrap_link on a Value variant"),
        }
    }
    pub async fn resolve_with<R>(self, resolver: &R) -> Result<V, R::Error>
    where
        R: Resolver<L, V>,
        L: Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        <Self as Resolvable<L, V, V>>::resolve_with(self, resolver).await
    }
}

impl<L, V: Default> Default for LinkOrValue<L, V> {
    fn default() -> Self {
        LinkOrValue::Value(V::default())
    }
}

pub trait Resolver<L, V>: Sized + Send + Sync + Clone + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    fn resolve(&self, link: L) -> impl Future<Output = Result<V, Self::Error>> + Send + '_;
}

pub trait Resolvable<L, V, T> {
    fn resolve_with<R: Resolver<L, V>>(
        self,
        resolver: &R,
    ) -> impl Future<Output = Result<T, R::Error>> + Send + '_;
}

impl<L, V> Resolvable<L, V, V> for LinkOrValue<L, V>
where
    L: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, V>>(self, resolver: &R) -> Result<V, R::Error> {
        match self {
            LinkOrValue::Link(link) => {
                let value = resolver.resolve(link).await?;
                Ok(value)
            }
            LinkOrValue::Value(value) => Ok(value),
        }
    }
}
