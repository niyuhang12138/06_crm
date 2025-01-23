use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, InAppMessage, SendRequest, SendResponse},
    NotificationService,
};
use tonic::Status;
use tracing::warn;
use uuid::Uuid;

impl Sender for InAppMessage {
    async fn send(self, scv: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();

        scv.sender.send(Msg::InApp(self)).await.map_err(|e| {
            warn!("Failed to send in-app message: {e:?}");
            Status::internal("Failed to send in-app message")
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for SendRequest {
    fn from(value: InAppMessage) -> Self {
        Self {
            msg: Some(Msg::InApp(value)),
        }
    }
}

impl InAppMessage {
    pub fn fake() -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            derive_id: Uuid::new_v4().to_string(),
            title: "Hello".to_string(),
            body: "Hello, world".to_string(),
        }
    }
}
