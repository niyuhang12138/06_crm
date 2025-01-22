use anyhow::Result;
use crm::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "ls".to_string(),
        email: "ls@163.com".to_string(),
    });

    let response = client.create_user(request).await?;
    println!("RESPONSE={:?}", response);

    let user = response.into_inner();
    println!("USER={:?}", user);

    Ok(())
}
