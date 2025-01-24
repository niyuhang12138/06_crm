use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use sqlx_db_tester::TestPg;
use tokio::time::sleep;
use tonic::transport::Server;
use user_stat::{
    tests_util::{id, tq},
    QueryRequestBuilder, RawQueryRequestBuilder, UserStatsClient, UserStatsService,
};

const PORT: u32 = 60000;

#[tokio::test]
async fn test_raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;

    let stream = client.raw_query(req).await?.into_inner();
    let users = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert!(!users.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}",)).await?;
    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(Some(450), None)))
        .timestamp(("last_visited_at".to_string(), tq(Some(400), None)))
        .id(("viewed_but_not_started".to_string(), id(&[252790])))
        .build()
        .unwrap();
    let stream = client.query(query).await?.into_inner();
    let users = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    println!("{}", users.len());
    assert_eq!(users.len(), 16);

    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;
    let (tdb, svc) = UserStatsService::new_for_test().await?;
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;
    Ok((tdb, addr))
}
