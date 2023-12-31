use std::io::{stdout, Write};

use anyhow::{anyhow, Result};
use atty::Stream;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use xdiff::{
    cli::{parse_key_val, KeyVal},
    utils::highlight,
    ExtraArgs, LoadYaml, ReqConfig, ReqProfile, RequestProfile,
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
        Action::Parse(_) => parse().await,
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

    let header_text = res.get_header_text(&profile.response.skip_headers)?;
    let body_text = res.get_body_text(&profile.response.skip_body).await?;

    let mut stdout = stdout().lock();

    if atty::is(Stream::Stdout) {
        writeln!(
            stdout,
            "------\n{}",
            highlight(&format!("{}{}", header_text, body_text), "json")?
        )?;
    } else {
        writeln!(stdout, "{}", body_text)?;
    }

    Ok(())
}

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url: String = Input::with_theme(&theme)
        .with_prompt("Please enter url")
        .interact_text()?;

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Please enter profile name")
        .interact_text()?;

    let request: RequestProfile = url.parse()?;

    let res = request.send(&ExtraArgs::default()).await?;

    let header_options = res.get_headers();
    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Choose headers to skip")
        .items(&header_options)
        .interact()?;

    let skip_headers = chosen
        .iter()
        .map(|&i| header_options[i].to_string())
        .collect::<Vec<_>>();

    let profile = ReqProfile::new(request, skip_headers);

    let config = ReqConfig::new(vec![(profile_name, profile)].into_iter().collect());

    let output = serde_yaml::to_string(&config)?;

    let mut stdout = stdout().lock();
    writeln!(stdout, "------\n{}", highlight(&output, "yaml")?)?;

    Ok(())
}
