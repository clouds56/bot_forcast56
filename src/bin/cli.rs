#[macro_use]
extern crate log;

use std::collections::HashSet;

use anyhow::Result;
use telegram_forcast56::onu;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
enum InfoTarget {
  Lan, Wan,
}

impl std::fmt::Display for InfoTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.to_possible_value()
        .expect("no values are skipped")
        .get_name()
        .fmt(f)
  }
}

#[derive(Debug, Subcommand)]
enum Commands {
  #[command(arg_required_else_help = true)]
  Info {
    // #[arg(value_enum)]
    target: InfoTarget,
  },
}

#[derive(Debug, Parser)]
struct Cli {
  #[arg(long, default_value = "http://192.168.1.1")]
  base_url: String,
  #[command(subcommand)]
  command: Commands,
}

async fn ctx(base_url: &str) -> Result<onu::Context> {
  let username = std::env::var("router_username").unwrap();
  let password = std::env::var("router_password").unwrap();
  let mut ctx = onu::Context::new(base_url);
  ctx.login(&username, &password).await?;
  Ok(ctx)
}

fn json_to_csv(json: &[serde_json::Value]) -> Result<Vec<csv::StringRecord>> {
  let mut headers = Vec::new();
  let mut header_set = HashSet::new();
  for row in json {
    for key in row.as_object().ok_or_else(|| anyhow::format_err!("not object"))?.keys() {
      if !header_set.contains(key) {
        headers.push(key.clone());
        header_set.insert(key.clone());
      }
    }
  }
  let mut records = vec![csv::StringRecord::from(headers.clone())];
  for row in json {
    let values = headers.iter().map(|header| {
      row.get(header).map(|v| match v.as_str() {
        Some(s) => s.to_string(),
        _ => v.to_string(),
      }).unwrap_or_default()
    }).collect::<Vec<_>>();
    records.push(csv::StringRecord::from(values));
  }
  Ok(records)
}

fn csv_to_table(records: &[csv::StringRecord]) -> tabled::Table {
  let mut builder = tabled::builder::Builder::new();
  // println!("{:?}", records[0]);
  for i in records.get(0).unwrap() {
    builder.push_record(&[i.to_string()]);
  }
  records.iter().skip(1).for_each(|i| builder.push_column(i));
  builder.build()
}

#[tokio::main]
async fn main() -> Result<()> {
  dotenvy::dotenv().ok();
  flexi_logger::Logger::try_with_env_or_str("info").unwrap().start().ok();
  let args = Cli::parse();
  info!("{:?}", args);
  match args.command {
    Commands::Info { target } => {
      let mut ctx = ctx(&args.base_url).await?;
      match target {
        InfoTarget::Lan => {
          let info = ctx.lan_info().await?;
          println!("{}", tabled::Table::new(&info));
        }
        InfoTarget::Wan => {
          let info = ctx.wan_info().await?;
          let info_json = info.iter().map(serde_json::to_value).collect::<Result<Vec<_>,_>>()?;
          println!("{}", csv_to_table(&json_to_csv(&info_json)?));
        }
      }
    }
  }
  Ok(())
}
