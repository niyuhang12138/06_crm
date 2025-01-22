mod abi;
mod config;
mod pb;

use anyhow::Result;
pub use config::AppConfig;
pub use pb::*;
use sqlx::PgPool;
use std::{ops::Deref, pin::Pin, sync::Arc};
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct UserStatsService {
    pub inner: Arc<UserStatsServiceInner>,
}

pub struct UserStatsServiceInner {
    pub config: AppConfig,
    pub pool: PgPool,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

#[tonic::async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    type RawQueryStream = ResponseStream;

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl From<UserStatsService> for UserStatsServer<UserStatsService> {
    fn from(value: UserStatsService) -> Self {
        UserStatsServer::new(value)
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to db");

        let inner = UserStatsServiceInner { config, pool };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        self.into()
    }
}

impl Deref for UserStatsService {
    type Target = Arc<UserStatsServiceInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
