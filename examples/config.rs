use xdiff::DiffConfig;

fn main() {
    let content = include_str!("../fixtures/test.yml");
    let config: DiffConfig = serde_yaml::from_str(content).unwrap();
    println!("{:#?}", config);
}
