use std::io::{stdout, Write};

use anyhow::{anyhow, Result};
use clap::Parser;
use xdiff::{
    cli::{parse_key_val, KeyVal},
    utils::highlight,
    ExtraArgs, LoadYaml, ReqConfig,
};

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
    let config_file = args.config.unwrap_or_else(|| "req.yaml".to_string());
    let config = ReqConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow!(
            "Profile {} not found in config file {}",
            args.profile,
            config_file
        )
    })?;
    let extra_args = ExtraArgs::from(args.extra_params);

    let res: xdiff::ResponseExt = profile.request.send(&extra_args).await?;
    let output = res
        .get_text(&profile.response.skip_headers, &profile.response.skip_body)
        .await?;

    let mut stdout = stdout().lock();

    writeln!(stdout, "------\n{}", highlight(&output, "json")?)?;

    Ok(())
}

async fn parse(args: ParseArgs) -> Result<()> {
    println!("parse: {:?}", args);

    Ok(())
}
