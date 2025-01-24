use anyhow::Result;
use crm_send::{
    pb::{notification_client::NotificationClient, EmailMessage, InAppMessage, SmsMessage},
    AppConfig, NotificationService,
};
use futures::StreamExt;
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
use tonic::{transport::Server, Request};

#[tokio::test]
async fn test_send_should_work() -> Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{addr}")).await?;
    let stream = tokio_stream::iter(vec![
        EmailMessage::fake().into(),
        SmsMessage::fake().into(),
        InAppMessage::fake().into(),
    ]);

    let request = Request::new(stream);
    let response = client.send(request).await?.into_inner();
    let ret = response
        .then(|res| async { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;
    let svc = NotificationService::new(config).into_server();
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;
    Ok(addr)
}
