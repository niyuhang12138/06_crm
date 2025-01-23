mod email;
mod in_app;
mod sms;

use crate::{
    pb::{notification_server::NotificationServer, send_request::Msg, SendRequest, SendResponse},
    AppConfig, NotificationInner, NotificationService, ResponseStream, ServiceResult,
};
use chrono::Utc;
use futures::Stream;
use prost_types::Timestamp;
use std::{ops::Deref, sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Response, Status};
use tracing::{info, warn};

const CHANNEL_SIZE: usize = 1024;

pub trait Sender {
    async fn send(self, scv: NotificationService) -> Result<SendResponse, Status>;
}

impl NotificationService {
    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        Self {
            inner: Arc::new(NotificationInner { config, sender }),
        }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }

    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let notify = self.clone();
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let notify_clone = notify.clone();
                let res = match req.msg {
                    Some(Msg::Email(email)) => email.send(notify_clone).await,
                    Some(Msg::Sms(sms)) => sms.send(notify_clone).await,
                    Some(Msg::InApp(in_app)) => in_app.send(notify_clone).await,
                    None => {
                        warn!("Invalid request");
                        Err(Status::invalid_argument("Invalid request"))
                    }
                };
                tx.send(res).await.unwrap();
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl Deref for NotificationService {
    type Target = NotificationInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

pub fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sending message: {:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pb::{EmailMessage, InAppMessage, SmsMessage},
        AppConfig,
    };
    use anyhow::Result;

    #[tokio::test]
    async fn send_msg_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let svc = NotificationService::new(config);
        let stream = tokio_stream::iter(vec![
            Ok(EmailMessage::fake().into()),
            Ok(SmsMessage::fake().into()),
            Ok(InAppMessage::fake().into()),
        ]);
        let response = svc.send(stream).await?;
        let ret = response.into_inner().collect::<Vec<_>>().await;
        assert_eq!(ret.len(), 3);
        Ok(())
    }
}
