use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    name: String,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

fn main() {
    let json = r#"{
        "name": "JiangBo",
        "page_number": 4,
        "page_size": 44,
        "total": 22
      }"#;

    let req: Request = serde_json::from_str(json).unwrap();
    println!("{:?}", req);
    println!("{}", serde_json::to_string(&req).unwrap());
}
