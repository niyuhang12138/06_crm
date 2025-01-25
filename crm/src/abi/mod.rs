use std::sync::Arc;

use chrono::{Duration, Utc};
use crm_metadata::pb::MaterializeRequest;
use crm_send::pb::SendRequest;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::warn;
use user_stat::QueryRequest;

use crate::{
    pb::{WelcomeRequest, WelcomeResponse},
    CrmService,
};

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);

        let query = QueryRequest::new_with_dt("created_at", d1, d2);

        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner();

        let contents = contents
            .filter_map(|v| async move { v.ok() })
            .collect::<Vec<_>>()
            .await;

        let contents = Arc::new(contents);

        let (tx, rx) = mpsc::channel(1024);

        let sender = self.config.server.sender_email.clone();

        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents_clone = contents.clone();
                let sender_clone = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new(
                    "Welusercome".to_string(),
                    sender_clone,
                    &[user.email],
                    &contents_clone,
                );

                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {e:?}");
                }
            }
        });

        let req = ReceiverStream::new(rx);

        self.notification.clone().send(req).await?;

        Ok(Response::new(WelcomeResponse { id: request_id }))
    }
}
