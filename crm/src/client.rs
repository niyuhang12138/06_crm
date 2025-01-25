use anyhow::Result;
use crm::pb::{crm_client::CrmClient, WelcomeRequestBuilder};
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let pem = include_str!("../../fixtures/rootCA.pem");
    let ca = Certificate::from_pem(pem);

    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("localhost");
    let channel = Channel::from_static("https://[::1]:50000")
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = CrmClient::new(channel);

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(92_u32)
        .content_ids([1_u32, 2, 3])
        .build()?;

    let response = client.welcome(Request::new(req)).await?.into_inner();

    println!("Response: {:?}", response);

    Ok(())
}
