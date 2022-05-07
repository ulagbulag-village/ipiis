use std::sync::Arc;

use bytecheck::CheckBytes;
use ipiis_api_quic::{
    client::IpiisClient,
    common::{opcode::Opcode, Ipiis},
    server::IpiisServer,
};
use ipis::{
    class::Class,
    core::{
        account::{AccountRef, GuaranteeSigned},
        anyhow::Result,
    },
    env::Infer,
    pin::Pinned,
};
use rkyv::{Archive, Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    // init peers
    let server = run_server(5001).await?;
    let client = run_client(server, 5001).await?;

    // create a data
    let req = Arc::new(Request {
        name: "Alice".to_string(),
        age: 42,
    });

    for _ in 0..5 {
        // recv data
        let res: GuaranteeSigned<String> = client
            .call_permanent_deserialized(Opcode::TEXT, &server, req.clone())
            .await?;

        // verify data
        assert_eq!(
            res.data.data,
            format!("hello, {} years old {}!", &req.name, req.age),
        );
    }
    Ok(())
}

async fn run_client(server: AccountRef, port: u16) -> Result<IpiisClient> {
    // init a client
    let client = IpiisClient::genesis(None)?;
    client.add_address(server, format!("127.0.0.1:{}", port).parse()?)?;
    Ok(client)
}

async fn run_server(port: u16) -> Result<AccountRef> {
    // init a server
    let server = IpiisServer::genesis(port)?;
    let public_key = server.account_me().account_ref();

    // accept a single connection
    let server = Arc::new(server);
    tokio::spawn(async move { server.run(server.clone(), handle).await });

    Ok(public_key)
}

#[derive(Class, Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct Request {
    name: String,
    age: u32,
}

async fn handle(
    _server: Arc<IpiisServer>,
    req: Pinned<GuaranteeSigned<Arc<Request>>>,
) -> Result<String> {
    // resolve data
    let req = &req.data.data;

    // handle data
    let res = format!("hello, {} years old {}!", &req.name, req.age);

    Ok(res)
}
