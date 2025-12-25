use crate::{LinkResolver, formats::PLAINTEXT};
#[derive(Clone, Debug, Default)]
pub struct EnvLinkResolver;

impl LinkResolver for EnvLinkResolver {
    async fn fetch<V: crate::formats::TransferObject>(
        &self,
        link: &super::Link,
    ) -> Result<crate::ConfigWithFormat<V>, crate::Error> {
        if let Some(var_name) = link.as_env_ame() {
            match std::env::var(var_name) {
                Ok(value) => {
                    let bytes = value.into_bytes();
                    crate::ConfigWithFormat::decode(PLAINTEXT, bytes.into())
                }
                Err(e) => Err(crate::Error::resolve_error(e, link.clone())),
            }
        } else {
            Err(crate::Error::resolve_error("Not an env link", link.clone()))
        }
    }
}
