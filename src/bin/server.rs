#![feature(impl_trait_in_assoc_type)]

use std::env;
use std::net::SocketAddr;
use volo_example::{FilterLayer, S};

//use volo_gen::volo::example::GetItemRequest;
// use lazy_static::lazy_static;
// use pilota::lazy_static;
//use std::io::Error;
//use std::io::Write;
// lazy_static! {
//     static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
//         let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
//         volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
//             .layer_outer(FilterLayer)
//             .address(addr)
//             .build()
//     };
// }

#[volo::main]
async fn main() {
    // tracing_subscriber::fmt::init();    
    let mut arg: Vec<String> = env::args().collect(); 

    /*
    run --bin server <host> <port> <slaves'addr>
    */
    if arg.len() < 3{
        panic!("command: run --bin server <host> <port> <slaves'addr>");
    }

    let host = arg.remove(1);
    let port = arg.remove(1);
    arg.remove(0);
    let sla_addr:Vec<_> = arg.clone().into_iter().map(|t|t.parse::<SocketAddr>().unwrap()).collect();
    
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    let addr = volo::net::Address::from(addr);
    
    let log_path = format!("log/{}_{}_{}",host, port, match sla_addr.is_empty() {
                                true=>"slave",
                                false=>"master"
                            });
    println!("log_path: {}", log_path.clone());

    volo_gen::volo::example::ItemServiceServer::new(S::new(sla_addr,log_path.as_str()).await)
        .layer_front(FilterLayer)
        .run(addr)
        .await
        .unwrap();
}