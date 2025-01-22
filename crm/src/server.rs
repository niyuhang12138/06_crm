use anyhow::Result;
use crm::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, GetUserRequest, User,
};
use tonic::{async_trait, transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct UserServer;

#[async_trait]
impl UserService for UserServer {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("create_user: {:?}", input);
        let user = User::new(1, &input.name, &input.email);
        Ok(Response::new(user))
    }

    async fn get_user_by_id(
        &self,
        request: tonic::Request<GetUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        let input = request.into_inner();
        println!("get_user_by_id: {:?}", input);
        Ok(Response::new(User::default()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse().unwrap();
    let svc = UserServer;

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
