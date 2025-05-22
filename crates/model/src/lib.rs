use std::collections::HashMap;

pub mod cursor;
pub use cursor::*;
pub mod descriptor;
pub use descriptor::*;
pub mod bind;
pub use bind::*;
pub mod tag;
pub use tag::*;
pub mod named_service;
pub use named_service::*;
use tokio::io::AsyncRead;

pub enum ConfigEvent {}

pub trait ConfigListener: Send {
    type Error: std::error::Error;
    fn next(&mut self) -> impl Future<Output = Result<ConfigEvent, Self::Error>> + Send + '_;
    fn update_subscription(
        &mut self,
        items: Vec<String>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;
}

pub trait ConfigService {
    type Error: std::error::Error;
    // fn get_many_binds(
    //     &self,
    //     query: BindQuery,
    //     cursor: CursorQuery,
    // ) -> impl Future<Output = Result<PagedResult<Bind>, Self::Error>> + Send + '_;
    // fn get_item_by_id(
    //     &self,
    //     id: String,
    // ) -> impl Future<Output = Result<Option<Bind>, Self::Error>> + Send + '_;
    // fn has_named_service(
    //     &self,
    //     name: String,
    // ) -> impl Future<Output = Result<bool, Self::Error>> + Send + '_;
    // fn add_items(
    //     &self,
    //     items: Vec<Bind>,
    // ) -> impl Future<Output = Result<Vec<Result<String, Self::Error>>, Self::Error>> + Send + '_;
    // fn delete_items(
    //     &self,
    //     ids: Vec<String>,
    // ) -> impl Future<Output = Result<Vec<Result<(), Self::Error>>, Self::Error>> + Send + '_;
    // fn update_items(
    //     &self,
    //     items: HashMap<String, Bind>,
    // ) -> impl Future<Output = Result<Vec<Result<(), Self::Error>>, Self::Error>> + Send + '_;
    fn get_named_service(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<NamedService>, Self::Error>> + Send + '_;
    // fn set_named_service_config(
    //     &self,
    //     name: String,
    //     config: impl AsyncRead + Send + 'static,
    // ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;
    // fn is_enabled(&self, id: String)
    // -> impl Future<Output = Result<bool, Self::Error>> + Send + '_;
    // fn set_enabled(
    //     &self,
    //     items: HashMap<String, bool>,
    // ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;
    fn get_enabled(
        &self,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<Bind>, Self::Error>> + Send + '_;

    // fn listen(
    //     &self,
    //     items: Vec<String>,
    // ) -> impl Future<Output = Result<impl ConfigListener, Self::Error>> + Send + '_;
}
