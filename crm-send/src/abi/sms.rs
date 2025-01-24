use super::{to_ts, Sender};
use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse, SmsMessage},
    NotificationService,
};
use fake::{faker::phone_number::zh_cn::PhoneNumber, Fake};
use tonic::Status;
use tracing::warn;
use uuid::Uuid;

impl Sender for SmsMessage {
    async fn send(self, scv: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();

        scv.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("Failed to send sms message: {e:?}");
            Status::internal("Failed to send sms message")
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(value: SmsMessage) -> Self {
        Self {
            msg: Some(Msg::Sms(value)),
        }
    }
}

#[cfg(feature = "test-util")]
impl SmsMessage {
    pub fn fake() -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            sender: PhoneNumber().fake(),
            recipients: vec![PhoneNumber().fake()],
            body: "Hello, world".to_string(),
        }
    }
}
