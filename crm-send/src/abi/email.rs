use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, EmailMessage, SendRequest, SendResponse},
    NotificationService,
};
use tonic::Status;
use tracing::warn;

impl Sender for EmailMessage {
    async fn send(self, scv: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();

        scv.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send email message: {e:?}");
            Status::internal("Failed to send email message")
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<EmailMessage> for SendRequest {
    fn from(value: EmailMessage) -> Self {
        Self {
            msg: Some(Msg::Email(value)),
        }
    }
}

#[cfg(feature = "test-util")]
impl EmailMessage {
    pub fn fake() -> Self {
        use fake::{faker::internet::zh_cn::SafeEmail, Fake};
        use uuid::Uuid;
        Self {
            message_id: Uuid::new_v4().to_string(),
            subject: "Hello".to_string(),
            sender: SafeEmail().fake(),
            recipients: vec![SafeEmail().fake()],
            body: "Hello, world".to_string(),
        }
    }
}
