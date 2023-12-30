use anyhow::Result;
use clap::Parser;
use xdiff::cli::{parse_key_val, KeyVal};

#[derive(Debug, Parser, Clone)]
#[clap(version, author, about, long_about)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Parser, Clone)]
pub enum Action {
    Run(RunArgs),
    Parse(ParseArgs),
}

#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    /// profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args. Could be used to voerride the query, headers and body of the request.
    /// for query params, use `-e key=value`
    /// for headers, use `-e %key=value`
    /// for body, use `-e @key=value`
    #[clap(short, long, value_parser = parse_key_val, number_of_values=1)]
    pub extra_params: Vec<KeyVal>,
    /// config file
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Debug, Parser, Clone)]
pub struct ParseArgs {}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Run(args) => run(args).await,
        Action::Parse(args) => parse(args).await,
    }
}

async fn run(args: RunArgs) -> Result<()> {
    println!("run: {:?}", args);
    Ok(())
}

async fn parse(args: ParseArgs) -> Result<()> {
    println!("parse: {:?}", args);

    Ok(())
}
