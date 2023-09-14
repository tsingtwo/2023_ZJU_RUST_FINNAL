use lazy_static::lazy_static;
use pilota::lazy_static;
use volo_gen::volo::example::GetItemRequest;
use std::env;
use std::io;
use std::net::SocketAddr;
use volo_example::FilterLayer;
//use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
//use std::io::prelude::*;
static mut N_ADDR: String = String::new();
lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        unsafe {
            let addr: SocketAddr = N_ADDR.parse().unwrap();
            volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
                .layer_outer(FilterLayer)
                .address(addr)
                .build()
        }
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
    let arg: Vec<String> = env::args().collect(); 
    if arg.len() > 1{
        unsafe {
            N_ADDR = arg[1].clone();
        }
    }
    let mut subs = false;
    let mut channel = String::new();
    loop {
        if subs{
            let subsresp = CLIENT.get_item(GetItemRequest { op: "subscribe".into(), key: channel.clone().into(), val: " ".into() }).await;
            match subsresp {
                Ok(info)=>{
                    println!("{}", info.val);
                },
                Err(e)=>tracing::error!("{:?}", e),
            }
            continue;
        }
        // let mut args: Vec<String> = std::env::args().collect();
        let mut a = String::new();
        io::stdin().read_line(&mut a).unwrap();
        let mut args: Vec<String> = a.trim().split(" ").map(|t| t.to_string()).collect();
        // if args.len() < 1{ continue; }
        
        let mut req = GetItemRequest { op: " ".into(), key: " ".into(), val: " ".into() };
        let opcode = args.remove(0).clone().to_lowercase().to_string();
        // println!("{}|{}|{}",opcode.clone(),args[0], args[1]);
        // println!("|{}|", opcode.clone());
        if opcode == "set".to_string() {
            if args.len() < 1{ continue; }
            req = GetItemRequest{
                op:"set".into(),
                key:args.remove(0).clone().into(),
                val:args.remove(0).clone().into(),
            };
            
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
        }else if opcode == "get".to_string() {
            if args.len() < 1{ continue; }
            req = GetItemRequest{
                op:"get".into(),
                key:args.remove(0).clone().into(),
                val:" ".into(),
            };
            // println!("{}, {}, {}", req.op, req.key, req.val);
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
        }else if opcode == "del".to_string() {
            if args.len() < 1{ continue; }
            req = GetItemRequest{
                op:"del".into(),
                key:args.remove(0).clone().into(),
                val:" ".into(),
            };
            
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
        }else if opcode == "ping".to_string() {
            req = GetItemRequest{
                op:"ping".into(),
                key:" ".into(),
                val:" ".into(),
            };
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
        }else if opcode == "subscribe".to_string() {
            if args.len() < 1{ continue; }
            subs = true;
            req = GetItemRequest{
                op:"subscribe".into(),
                key:args.remove(0).clone().into(),
                val:" ".into(),
            };
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
            channel = req.clone().key.to_string();
        }else if opcode == "publish".to_string() {
            if args.len() < 1{ continue; }
            req = GetItemRequest{
                op:"publish".into(),
                key:args.remove(0).clone().into(),
                val:args.remove(0).clone().into(),
            };
            // println!("{}, {}, {}", req.clone().op, req.clone().key, req.clone().val);
        }else{
            println!("{} is Unkown command", opcode);
        }
        let k = req.clone().key;
        let v= req.clone().val;
        let resp = CLIENT.get_item(req).await;
        match resp {
            Ok(info)=>{
                if info.op=="set".to_string() {
                    if info.status{
                        println!("SET SUCCESS");
                        
                    } else {
                        println!("ALREADY EXISTED");
                    }
                } else if info.op=="get".to_string() {
                    if info.status{
                        println!("GET SUCCESS, {}", info.val);
                    } else {
                        println!("NOT FOUUND");
                    }
                } else if info.op=="del".to_string() {
                    if info.status{
                        println!("DEL SUCCESS");
                        
                    } else {
                        println!("NOT FOUUND");
                    }
                } else if info.op=="ping".to_string() {
                    if info.status{
                        println!("pong");
                    } else {
                        println!("FAILED");
                    }
                }else if info.op=="subcribe".to_string() {
                    if info.status{
                        println!("{}", info.val);
                    } else {
                        println!("NO PUBLISH");
                    }
                } else if info.op=="publish".to_string() {
                    let msg = info.val.clone().to_string();
                    if info.status{
                        println!("THE NUMBERR IS {}", msg);
                    } else {
                        println!("NOT FOUND");
                    }
                }
                // let mut f = OpenOptions::new()
                //             .read(true)
                //             .write(true)
                //             .create(true)
                //             .append(true)
                //             .open("LogFile.txt").unwrap();
                // let _addr = "0:0:0:0".to_string();
                // f.write_all(info.op.as_bytes()).expect("write failed");
                // f.write_all(",".as_bytes()).expect("write failed");
                // f.write_all(k.as_bytes()).expect("write failed");
                // f.write_all(",".as_bytes()).expect("write failed");
                // f.write_all(v.as_bytes()).expect("write failed");
                // f.write_all(",".as_bytes()).expect("write failed");
                // f.write_all(_addr.as_bytes()).expect("write failed");
                // f.write_all("\n".as_bytes()).expect("write failed");
            },
            Err(e) => tracing::error!("{:?}", e),
        }
    }
}