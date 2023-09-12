#![feature(impl_trait_in_assoc_type)]
use std::{collections::HashMap, sync::Mutex};

pub struct S{
	kav: Mutex<HashMap<String,String>>
}
impl S {
	pub fn new()->S{
		S { kav: Mutex::new(HashMap::new()) }
	}
}
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
				resp.status = false;
			}
		}else if _req.op == "get".to_string() {
			resp.op = "get".to_string().into();
			let k = _req.key.to_string();
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
				}
				None=>{
					println!("del2.2");
					resp.status = false;
				}	
			}
		}else if _req.op == "ping".to_string(){
			println!("ping1");
			resp.op = "ping".to_string().into();
			resp.status = true;
			println!("ping2");
		}else {
			panic!("INVALID OP! ");
		}
		Ok(resp)
    }
}
