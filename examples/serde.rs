use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "request1")]
struct Paging {
    page_number: usize,
    page_size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    name: String,
    paging: Paging,
}

fn main() {
    let request: Request = Request {
        name: "JiangBo".to_owned(),
        paging: Paging {
            page_number: 4,
            page_size: 44,
        },
    };

    let json = serde_json::to_string(&request).unwrap();
    println!("{}", json);
    let req: Request = serde_json::from_str(&json).unwrap();
    println!("{:?}", req);

    let mut json = json!({
        "name": "JiangBo",
        "paging": {
            "page_number": 4,
            "page_size": 44
        }
    });

    json["name"] = json!("john");
    println!("{} \n{}", json["name"], json.to_string());
}
