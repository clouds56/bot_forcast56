#[macro_use]
extern crate log;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  flexi_logger::Logger::try_with_env_or_str("info").unwrap().start().ok();
  let args = Cli::parse();
  info!("{:?}", args);
  match args.command {
    Commands::Info { target } => {
      let mut ctx = onu::Context::new(&args.base_url);
      match target {
        InfoTarget::Lan => {
          let info = ctx.lan_info().await?;
          println!("{:?}", info);
        }
        InfoTarget::Wan => {
          let info = ctx.wan_info().await?;
          println!("{:?}", info);
        }
      }
    }
  }
  Ok(())
}
