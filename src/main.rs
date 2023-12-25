use std::io::{stdout, Write};

use anyhow::{anyhow, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use xdiff::{
    cli::{Action, Args, RunArgs},
    DiffConfig, DiffProfile, ExtraArgs, RequestProfile,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse(_) => parse().await?,
        _ => panic!("Unsupported action"),
    }

    Ok(())
}

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url1: String = Input::with_theme(&theme)
        .with_prompt("Please enter url1")
        .interact_text()?;

    let url2: String = Input::with_theme(&theme)
        .with_prompt("Please enter url2")
        .interact_text()?;

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Please enter profile name")
        .interact_text()?;

    let req1: RequestProfile = url1.parse()?;
    let req2: RequestProfile = url2.parse()?;

    let res1 = req1.send(&ExtraArgs::default()).await?;
    let res2 = req2.send(&ExtraArgs::default()).await?;

    let header_options = [res1.get_headers(), res2.get_headers()].concat();
    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Choose headers to skip")
        .items(&header_options)
        .interact()?;

    let skip_headers = chosen
        .iter()
        .map(|&i| header_options[i].to_string())
        .collect::<Vec<_>>();

    let profile = DiffProfile::new(req1, req2, skip_headers);
    let config: DiffConfig = DiffConfig::new(vec![(profile_name, profile)].into_iter().collect());

    let output = serde_yaml::to_string(&config)?;

    let mut stdout = stdout().lock();
    writeln!(stdout, "------\n{}", output)?;

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
