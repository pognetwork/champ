mod rpc;

use clap::clap_app;
use rpc::server::RpcServer;

#[tokio::main]
async fn main() {
    let matches = clap_app!("champ-node" =>
        (version: "0.0.1")
        (author: "The POG Project <contact@pog.network>")
        (about: "POG's reference implementation in rust")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
    )
    .get_matches();

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    let addr = "[::1]:50051".parse().unwrap();
    let addr2 = "[::1]:50050".parse().unwrap();

    let _ = tokio::try_join!(RpcServer::start(addr), RpcServer::start(addr2));
}
