mod abi;
mod config;
pub mod pb;

pub use abi::{DecodingKey, User};
use anyhow::Result;
pub use config::AppConfig;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{
    service::interceptor::InterceptedService, transport::Channel, Request, Response, Status,
};
use tracing::info;
use user_stat::UserStatsClient;

pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[tonic::async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let user: &User = request.extensions().get::<User>().unwrap();
        info!("user: {user:?}");
        self.welcome(request.into_inner()).await
    }

    async fn recall(
        &self,
        _request: Request<RecallRequest>,
    ) -> Result<Response<RecallResponse>, Status> {
        unimplemented!()
    }

    async fn remind(
        &self,
        _request: Request<RemindRequest>,
    ) -> Result<Response<RemindResponse>, Status> {
        unimplemented!()
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }

    pub fn into_server(self) -> Result<InterceptedService<CrmServer<CrmService>, DecodingKey>> {
        let dk = DecodingKey::load(&self.config.auth.pk)?;
        Ok(CrmServer::with_interceptor(self, dk))
    }
}
