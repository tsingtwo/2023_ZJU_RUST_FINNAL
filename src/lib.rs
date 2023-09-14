#![feature(impl_trait_in_assoc_type)]
use std::{collections::HashMap, sync::Mutex};
use anyhow::anyhow;
use pilota::FastStr;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use core::result::Result;
pub struct S{
	kav: Mutex<HashMap<String,String>>,
	pub channels: Mutex<HashMap<String, Sender<String>>>,
}
impl S {
	pub fn new()->S{
		// let mut f = OpenOptions::new()
        //                      .read(true)
        //                     .write(true)
        //                     .create(true)
        //                     .append(true)
        //                     .open("LogFile.txt").unwrap();
		// let s = f.read_line().split(" ").map(|t| t.to_string()).collect();

		S { kav: Mutex::new(HashMap::new()), channels:Mutex::new(HashMap::new())}
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
		if _req.op == "set".to_string(){
			resp.op = "set".to_string().into();
			let k = _req.key.to_string();
			let v = _req.val.to_string();
			println!("set {}, {}", k.clone(), v.clone());
			let  mut is_exist = true;
			if self.kav.lock().unwrap().get(&k) == None {
				is_exist = false;
			}
			if !is_exist{
				self.kav.lock().unwrap().insert(k, v);
				resp.status = true;
			}else{
				// resp.status = false;
				self.kav.lock().unwrap().remove(&k);
				self.kav.lock().unwrap().insert(k, v);
				resp.status = true;
			}
		}else if _req.op == "get".to_string() {
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
		}else if _req.op == "del".to_string(){
			println!("del1");
			resp.op = "del".to_string().into();
			let k = _req.key.to_string();
			match self.kav.lock().unwrap().remove(&k) {
				Some(t)=>{
					println!("del2.1.1");
					resp.status = true;
				},
				None=>{
					println!("del2.2");
					resp.status = false;
				}	
			}
		}else if _req.op == "ping".to_string(){
			resp.op = "ping".to_string().into();
			resp.status = true;
		}else if _req.op == "subscribe".to_string(){
			println!("here0");
			let k = _req.key.to_string();
			let (mut tx, mut rx) = broadcast::channel(16);
			let mut is_exist = true;
			resp.op = "subscribe".to_string().into();
			println!("here1");
			// match self.channels.lock().unwrap().get(&k){
			// 	Some(tx)=>{
			// 		rx = tx.subscribe();
			// 	},
			// 	None=>{
			// 		is_exist = false;
			// 	}
			// }
			if let Some(tx) =  self.channels.lock().unwrap().get(&k)  {
				rx = tx.subscribe();
				
			} else {
				is_exist = false;
				
			}
			println!("here2");
			if is_exist{
				println!("here3.0");
				let mes = rx.recv().await;
				println!("here3.0.1");
				match  mes {
					Ok(t)=>{
						resp.val = t.clone().into();
						resp.status = true;
					},
					Err(e)=>{
						resp.status = false; 
					}
				}
				println!("here3.0.2");
			}else {
				println!("here3.1");
				self.channels.lock().unwrap().insert(k, tx);
				println!("here3.1.0");
				let mes = rx.recv().await;
				println!("here3.1.1");
				match mes {
					Ok(t)=>{
						resp.val = t.clone().into();
						resp.status = false;
					},
					Err(e) => {
						resp.status = false;
					}
				}
				println!("here3.1.2");
			}
			
		}else if _req.op == "publish".to_string(){
			resp.op = "publish".to_string().into();
			let k = _req.key.to_string();
			match self.channels.lock().unwrap().get(&k) {
				Some(tx)=>{
					match tx.send(_req.val.to_string()) {
						Ok(n) => {
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
		}else {
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
		if info.contains("尊尼获嘉") {
			ill = false;
		} 
		if ill {
			let resp =self.0.call(cx, req).await;
			resp
		}else {
			Err(anyhow!("给你房管你给我说话").into())
		}
    }
}
