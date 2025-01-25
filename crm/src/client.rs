use anyhow::Result;
use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::Request;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("http://[::1]:{}", config.server.port);

    let mut client = CrmClient::connect(addr).await?;

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(92_u32)
        .content_ids([1_u32, 2, 3])
        .build()?;

    let response = client.welcome(Request::new(req)).await?.into_inner();

    println!("Response: {:?}", response);

    Ok(())
}
