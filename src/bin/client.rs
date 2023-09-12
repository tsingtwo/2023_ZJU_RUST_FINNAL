use lazy_static::lazy_static;
use pilota::lazy_static;
use volo_gen::volo::example::GetItemRequest;
use std::net::SocketAddr;
use volo_example::FilterLayer;
lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(FilterLayer)
            .address(addr)
            .build()
    };
}

#[volo::main]
#[allow(unused)]
async fn main() {
    /*
    cargo run --bin client <command> (<key>) (<val>)
    set zju 3
    set zju 1
    get sjtu
    get zju
    del sjtu
    del zju
    ping
     */
    tracing_subscriber::fmt::init();    
    let mut args: Vec<String> = std::env::args().collect();
    let mut req = GetItemRequest { op: " ".into(), key: " ".into(), val: " ".into() };

    let opcode = args.remove(1).clone().to_lowercase().to_string();
    if opcode == "set".to_string() {
        req = GetItemRequest{
            op:"set".into(),
            key:args.remove(1).clone().into(),
            val:args.remove(1).clone().into(),
        };
        println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
    }else if opcode == "get".to_string() {
        req = GetItemRequest{
            op:"get".into(),
            key:args.remove(1).clone().into(),
            val:" ".into(),
        };
        // println!("{}, {}, {}", req.op, req.key, req.val);
        println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
    }else if opcode == "del".to_string() {
        req = GetItemRequest{
            op:"del".into(),
            key:args.remove(1).clone().into(),
            val:" ".into(),
        };
        println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
    }else if opcode == "ping".to_string() {
        req = GetItemRequest{
            op:"ping".into(),
            key:" ".into(),
            val:" ".into(),
        };
        println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
    }else{
        println!("Unkown command");
    }

    let resp = CLIENT.get_item(req).await;
    match resp {
        Ok(info)=>{
            if info.op=="set".to_string(){
                if info.status{
                    println!("SET SUCCESS");
                }else {
                    println!("ALREADY EXISTED");
                }
            }
            else if info.op=="get".to_string() {
                if info.status{
                    println!("GET SUCCESS, {}", info.val);
                }else {
                    println!("NOT FOUUND");
                }
            }else if info.op=="del".to_string() {
                if info.status{
                    println!("DEL SUCCESS");
                }else {
                    println!("NOT FOUUND");
                }
            }else if info.op=="ping".to_string() {
                if info.status{
                    println!("pong");
                }else {
                    println!("FAILED");
                }
            }
            
        },
        Err(e) => tracing::error!("{:?}", e),
    }
    
}
