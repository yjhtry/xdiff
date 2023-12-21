use std::io::{stdout, Write};

use anyhow::{anyhow, Result};
use clap::Parser;
use xdiff::{
    cli::{Action, Args, RunArgs},
    DiffConfig, ExtraArgs,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("Unsupported action"),
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "xdiff.yaml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow!(
            "Profile {} not found in config file {}",
            args.profile,
            config_file
        )
    })?;
    let extra_args = ExtraArgs::from(args.extra_params);

    let output = profile.diff(extra_args).await?;

    let mut stdout = stdout().lock();

    stdout.write_all(output.as_bytes())?;

    Ok(())
}
