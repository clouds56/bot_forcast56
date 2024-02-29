#[macro_use]
extern crate log;

use std::{collections::HashSet, str::FromStr, path::PathBuf};

use anyhow::Result;
use telegram_forcast56::onu::{self, PortForwardingHost, PortForwardingParam, PortForwardingProtocol};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
enum InfoTarget {
  Lan, Wan, #[clap(name = "upnp")] UPnP, Wanc,
}

impl std::fmt::Display for InfoTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.to_possible_value()
        .expect("no values are skipped")
        .get_name()
        .fmt(f)
  }
}

/// Port range, possible values: 3389, 8000:8999
#[derive(Debug, Clone, Copy)]
struct PortRange(u32, u32);

impl FromStr for PortRange {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
      let mut iter = s.splitn(2, ':');
      let min = iter.next().unwrap().parse()?;
      let max = iter.next().map(str::parse::<u32>).transpose()?.unwrap_or(min);
      Ok(Self(min, max))
    }
}

impl std::fmt::Display for PortRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 == self.1 {
      self.0.fmt(f)
    } else {
      write!(f, "{}:{}", self.0, self.1)
    }
  }
}

#[derive(Debug, Subcommand)]
enum Commands {
  #[command(arg_required_else_help = true)]
  Info {
    target: InfoTarget,
  },
  #[command(arg_required_else_help = true)]
  PortForwarding {
    #[command(subcommand)]
    action: PortForwardingAction
  }
}

#[derive(Debug, Clone, Subcommand)]
enum PortForwardingAction {
  #[command(arg_required_else_help = true)]
  New {
    #[arg(long, name = "PORT")]
    external_port: Option<PortRange>,
    #[arg(long)]
    wanc: Option<String>,
    #[arg(long)]
    protocol: Option<PortForwardingProtocol>,
    name: String,
    #[arg(help = "local address, possible values: 192.168.1.4")]
    addr: PortForwardingHost,
    #[arg(help = "local port, possible values: 3389, 8000:8999")]
    port: PortRange,
  },
  #[command(arg_required_else_help = true)]
  Delete {
    name_or_index: String,
  },
}

#[derive(Debug, Parser)]
struct Cli {
  #[arg(long, default_value = "http://192.168.1.1")]
  base_url: String,
  #[arg(long)]
  cache_path: Option<PathBuf>,
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
      ctx.cache_path = args.cache_path;
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
        InfoTarget::UPnP => {
          let info = ctx.port_forwarding_list().await?;
          let view = info.into_iter().map(PortForwardingParamView::from).collect::<Vec<_>>();
          println!("{}", tabled::Table::new(&view));
        }
        InfoTarget::Wanc => {
          let info = ctx.wanc_info().await?;
          println!("{}", tabled::Table::new(&info));
        }
      }
    },
    Commands::PortForwarding { action } => {
      match action {
        PortForwardingAction::New { external_port,wanc,name,addr,port,protocol } => {
          let external_port = external_port.unwrap_or(port);
          let protocol = protocol.unwrap_or(PortForwardingProtocol::Both);
          // let name = name.unwrap_or_else(|| format!("__port_{}", port.0));
          let mut ctx = ctx(&args.base_url).await?;
          let wanc = match wanc {
            Some(wanc) => wanc,
            None => {
              let info = ctx.wanc_info().await?;
              info[0].view_name.clone()
            }
          };
          ctx.port_forwarding(
            onu::PortForwardingAction::New, name.as_str(), protocol, &wanc, addr,
            onu::PortForwardingPort::Multiple { remote: (external_port.0, external_port.1), local: (port.0, port.1) }).await?;
        },
        PortForwardingAction::Delete { name_or_index } => {
          let action = match name_or_index.parse::<u32>().ok() {
            Some(index) => onu::PortForwardingAction::Delete(index),
            None => onu::PortForwardingAction::DeleteByName(name_or_index),
          };
          let mut ctx = ctx(&args.base_url).await?;
          ctx.port_forwarding_delete(action).await?;
        },
      }
    }
  }
  Ok(())
}

#[derive(Debug, tabled::Tabled)]
struct PortForwardingParamView {
  enabled: bool,
  name: String,
  protocol: String,
  external_port: String,
  internal_port: String,
  internal_addr: String,
  description: String,
}
impl From<PortForwardingParam> for PortForwardingParamView {
  fn from(value: PortForwardingParam) -> Self {
    Self {
      name: value.name,
      enabled: value.enable,
      protocol: value.protocol.to_string(),
      external_port:
        if value.remote_port_min == value.remote_port_max { value.remote_port_min.to_string() }
        else { format!("{}:{}", value.remote_port_min, value.remote_port_max) },
      internal_port:
        if value.local_port_min == value.local_port_max { value.local_port_min.to_string() }
        else { format!("{}:{}", value.local_port_min, value.local_port_max) },
      internal_addr: if value.enable_local_mac { value.local_mac } else { value.local_addr }.unwrap_or_default(),
      description: value.description.unwrap_or_default(),
    }
  }
}
