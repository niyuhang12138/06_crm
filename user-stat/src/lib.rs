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

#[cfg(feature = "test-util")]
pub mod tests_util {
    use std::{env, path::Path, sync::Arc};

    use crate::{AppConfig, IdQuery, TimeQuery, UserStatsService, UserStatsServiceInner};
    use anyhow::Result;
    use chrono::Utc;
    use prost_types::Timestamp;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    impl UserStatsService {
        pub async fn new_for_test() -> Result<(TestPg, Self)> {
            let config = AppConfig::load()?;
            let post = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..post];
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
            let svc = Self {
                inner: Arc::new(UserStatsServiceInner { config, pool }),
            };
            Ok((tdb, svc))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };
        let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("migrations");
        let tdb = TestPg::new(url, path);
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/data.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }

    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }

    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }
    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
