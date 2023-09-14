#![feature(impl_trait_in_assoc_type)]
use std::path;

use std::net::SocketAddr;
use std::ptr::eq;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use anyhow::{anyhow, Error, Ok};
use pilota::FastStr;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use core::result::Result;
pub struct S{
	pub kav: Mutex<HashMap<String,String>>,
	pub channels: Mutex<HashMap<String, Sender<String>>>,
	pub slaves: bool,
	pub op_tx:Option<Arc<Mutex<broadcast::Sender<volo_gen::volo::example::GetItemRequest>>>>,
	pub log_file: Arc<tokio::sync::Mutex<File>>,
	// 对不起，这里一开始想得是slave的vector后来发现好像确实不是很需要
	
}
impl S {
	pub async fn new(slave_addr: Vec<SocketAddr>, path: &str)->S{
		let kav = Mutex::new(HashMap::new());
		let is_m = !slave_addr.is_empty();
		let op_tx = match is_m {
			true => Some(Arc::new(Mutex::new(broadcast::channel(16).0))),
			false => None,
		};
		if !is_m {
			println!("This is slave");
		}
		

		if !path::Path::new(&path).exists(){
			std::fs::create_dir_all("log").unwrap();
		}
		let log_file = OpenOptions::new()
										 .create(true)
										 .read(true)
										 .append(true)
										 .open(path)
										 .await.unwrap();

		let log_file = Arc::new(tokio::sync::Mutex::new(log_file));
		let mut buf = String::new();
		let _ = log_file.clone().lock().await.read_to_string(&mut buf).await;
		let contents: Vec<String> = buf.trim()
			.split("\n").map(|t| t.to_string()).collect();
		for s in contents {
			let cmd: Vec<String> = s.trim().split(" ").map(|t| t.to_string()).collect();
			if &cmd[0] == "set" {
				let mut is_exist = true;
				if kav.lock().unwrap().get(&cmd[1]) == None {
					is_exist = false;
				}
				if !is_exist {
					kav.lock().unwrap().insert(cmd[1].clone(), cmd[2].clone());
				} else if *kav.lock().unwrap().get(&cmd[1]).unwrap() != cmd[2] {
					kav.lock().unwrap().remove(&cmd[1]);
					kav.lock().unwrap().insert(cmd[1].clone(), cmd[2].clone());
				}
			} else if &cmd[0] == "del" {
				kav.lock().unwrap().remove(&cmd[1]);
			}
		}
		if is_m{
			for i in slave_addr{
				let operation_rx = Arc::new(tokio::sync::Mutex::new(op_tx.as_ref().unwrap().lock().unwrap().subscribe()));
				println!("{}", i);
				tokio::spawn(S::sync_slave(i, operation_rx));
			}
		}

		S { kav,
			channels:Mutex::new(HashMap::new()), 
			slaves: is_m, 
			op_tx, 
			log_file}
		
	}
	pub async fn sync_slave(
		slave_addr: SocketAddr,
		rx: Arc<tokio::sync::Mutex<broadcast::Receiver<volo_gen::volo::example::GetItemRequest>>>,
	)->Result<(), Error>{
		println!("DETECTED");
		let slave = RedisClient::new(slave_addr);
		loop {
			let req = rx.lock().await.recv().await;
			println!("DETECTED");
			match req {
				anyhow::Result::Ok(req)=>{
					println!("{} {} {}",req.op.clone(), req.key.clone(), req.val.clone());
					let resp = slave.get_item(req).await?;
					tracing::info!("{:?}", resp);
				},
				Err(e)=>{
					tracing::error!("{:?}", e);
				}
			}
		}
	}
}

pub struct RedisClient{
	client: volo_gen::volo::example::ItemServiceClient,
}
impl RedisClient {
	pub fn new(addr: SocketAddr)-> RedisClient{
		RedisClient{
			client:{
				volo_gen::volo::example::ItemServiceClientBuilder::new("volo_example")
				.address(addr)
				.build()
			}
		}
	}
	pub async fn get_item(
		&self,
		req: volo_gen::volo::example::GetItemRequest)-> 
		core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError>{
			match self.client.get_item(req).await {
				anyhow::Result::Ok(resp)=>{
					Ok(resp)
				},
				Err(e)=>{
					Err(Error::from(e))
				}
			}
		}
}

#[allow(unused)]
#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
    // 这部分是我们需要增加的代码
    async fn get_item(
        &self,
        _req: volo_gen::volo::example::GetItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError>
    {
		let mut resp = volo_gen::volo::example::GetItemResponse{op: " ".into(),key: " ".into(), val: " ".into(), status: false};
        // println!("{}, {}, {}", _req.op.clone(), _req.key.clone(), _req.val.clone());
		if _req.op == "set".to_string() || _req.op == "setfm".to_string(){
			resp.op = "set".to_string().into();
			let k = _req.key.to_string();
			let v = _req.val.to_string();
			println!("set {}, {}", k.clone(), v.clone());
			let mut is_exist = true;
			if self.kav.lock().unwrap().get(&k) == None {
				is_exist = false;
			}
			if self.slaves || _req.op == "setfm".to_string(){
				if !is_exist {
					self.kav.lock().unwrap().insert(k, v);
					resp.status = true;
					let _ = self.log_file.lock().await.write_all(format!("set {} {}\n",_req.key,_req.val).as_bytes()).await;
					if let Some(tx) = self.op_tx.clone() {
						let req = volo_gen::volo::example::GetItemRequest{op: "setfm".to_string().into(), key: _req.key.clone(), val: _req.val.clone()};
						let _ = tx.lock().unwrap().send(req);
					}
				} else if *self.kav.lock().unwrap().get(&k).unwrap() != v.to_string() {
					self.kav.lock().unwrap().remove(&k);
					self.kav.lock().unwrap().insert(k, v);
					resp.status = true;
					let _ = self.log_file.lock().await.write_all(format!("setfm {} {}\n",_req.key,_req.val).as_bytes()).await;
				} else {
					resp.status = false;
				}
			}else{
				resp.status = false;
			}
		} else if _req.op == "get".to_string() {
			resp.op = "get".to_string().into();
			let k = _req.key.to_string();
			println!("get {}", k.clone());
			match self.kav.lock().unwrap().get(&k) {
				Some(t)=>{
					resp.val = t.clone().into();
					resp.status = true;
				}
				None=>{
					resp.status = false;
				}
			}
		} else if _req.op == "del".to_string() || _req.op == "delfm".to_string(){
			//println!("del1");
			resp.op = "del".to_string().into();
			let k = _req.key.to_string();
			let is_exi = self.kav.lock().unwrap().contains_key(&k);
			if self.slaves || _req.op == "delfm".to_string(){
				match is_exi {
					true=>{
						println!("del1");
						resp.status = true;
						let _ = self.log_file.lock().await.write_all(format!("del {}\n",k.clone()).as_bytes()).await;
						self.kav.lock().unwrap().remove(&k);
						if let Some(tx) = self.op_tx.clone() {
							let req = volo_gen::volo::example::GetItemRequest{op: "delfm".to_string().into(), key: _req.key.clone(), val: _req.val.clone()};
							let _ = tx.lock().unwrap().send(req);
						}
					},
					false=>{
						println!("del2");
						resp.status = false;
					}	
				}
			}
		} else if _req.op == "ping".to_string(){
			resp.op = "ping".to_string().into();
			resp.status = true;
		} else if _req.op == "subscribe".to_string(){
			// println!("here0");
			let k = _req.key.to_string();
			let (mut tx, mut rx) = broadcast::channel(16);
			let mut is_exist = true;
			resp.op = "subscribe".to_string().into();
			if let Some(tx) =  self.channels.lock().unwrap().get(&k)  {
				rx = tx.subscribe();
				
			} else {
				is_exist = false;
				
			}
			if is_exist{
				let mes = rx.recv().await;
				match  mes {
					anyhow::Result::Ok(t)=>{
						resp.val = t.clone().into();
						resp.status = true;
					},
					Err(e)=>{
						resp.status = false; 
					}
				}
			} else {
				self.channels.lock().unwrap().insert(k, tx);
				let mes = rx.recv().await;
				match mes {
					anyhow::Result::Ok(t)=>{
						resp.val = t.clone().into();
						resp.status = false;
					},
					Err(e) => {
						resp.status = false;
					}
				}
			}
			
		} else if _req.op == "publish".to_string(){
			resp.op = "publish".to_string().into();
			let k = _req.key.to_string();
			match self.channels.lock().unwrap().get(&k) {
				Some(tx)=>{
					match tx.send(_req.val.to_string()) {
						anyhow::Result::Ok(n) => {
							resp.status = true;
							resp.val = FastStr::from((n as u8).to_string());
						},
						Err(_) => {
							resp.status = false;
						}
					}
				},
				None=>{
					resp.status = false;
				}
			}
		} else {
			panic!("INVALID OP! ");
		}
		Ok(resp)
    }
}

pub struct FilterLayer;
impl<S> volo::Layer<S> for FilterLayer {
    type Service = FilterService<S>;

    fn layer(self, inner: S) -> Self::Service {
        FilterService(inner)
    }
}
#[derive(Clone)]
pub struct FilterService<S>(S);
#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for FilterService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    Cx: Send + 'static,
	anyhow::Error: Into<S::Error>,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let info = format!("{req:?}");
		let mut ill = true;
		if info.contains("尊尼获嘉") || info.contains("Dell") {
			ill = false;
		} 
		if ill {
			let resp =self.0.call(cx, req).await;
			resp
		} else if info.contains("尊尼获嘉") {
			Err(anyhow!("给你房管你给我说话").into())
		} else {
			Err(anyhow!("Dell is shit").into())
		}
    }
}