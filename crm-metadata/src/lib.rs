mod abi;
mod config;
pub mod pb;

pub use config::AppConfig;
use futures::Stream;
use pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use std::pin::Pin;
use tonic::{Response, Status, Streaming};

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

#[tonic::async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: tonic::Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let stream = request.into_inner();
        self.materialize(stream).await
    }
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}
