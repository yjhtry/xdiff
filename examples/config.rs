use xdiff::ReqConfig;

fn main() {
    let content = include_str!("../fixtures/xreq.yml");
    let config: ReqConfig = serde_yaml::from_str(content).unwrap();
    println!("{:#?}", config);
}
