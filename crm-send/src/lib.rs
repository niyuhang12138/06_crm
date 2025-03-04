mod abi;
mod config;
pub mod pb;

use std::{pin::Pin, sync::Arc};

pub use config::AppConfig;
use futures::Stream;
use pb::{notification_server::Notification, send_request::Msg, SendRequest, SendResponse};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct NotificationService {
    pub inner: Arc<NotificationInner>,
}

pub struct NotificationInner {
    pub config: AppConfig,
    pub sender: mpsc::Sender<Msg>,
}

type ServiceResult<T> = Result<Response<T>, tonic::Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[tonic::async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<Self::SendStream> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}
