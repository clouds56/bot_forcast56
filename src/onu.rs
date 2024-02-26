use std::collections::HashMap;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Session {
  /// parsing from line
  /// `function getURL(){var ret = "getpage.gch?pid=1002&nextpage=";`
  url_next: String,
  /// parsing from line
  /// `var session_token = "859208547885....";`
  session_token: String,
}

mod serde_str01_as_bool {
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn serialize<S: Serializer>(b: &bool, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(if *b { "1" } else { "0" })
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<i32>().map_err(serde::de::Error::custom)? != 0)
  }
}

mod serde_str_as_u32 {
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn serialize<S>(b: &u32, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&b.to_string())
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(s.parse().map_err(serde::de::Error::custom)?)
  }
}

mod serde_strnull_as_option {
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn serialize<S: Serializer>(b: &Option<String>, serializer: S) -> Result<S::Ok, S::Error> {
    match b {
      Some(s) => serializer.serialize_str(s),
      None => serializer.serialize_str("NULL"),
    }
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
      "NULL" => Ok(None),
      _ => Ok(Some(s)),
    }
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WanIpInfo {
  /// NAT: 启用
  pub nat: String,
  /// IP: 0.0.0.0
  pub ip: String,
  /// DNS1: 0.0.0.0
  pub dns1: String,
  /// DNS2: 0.0.0.0
  pub dns2: String,
  /// DNS3: 0.0.0.0
  pub dns3: String,
  /// WAN MAC: xx:xx:xx:xx:xx:xx
  pub mac: String,
  /// 网关: 0.0.0.0
  pub gateway: String,
}

impl WanIpInfo {
  pub fn from_dict(dict: &HashMap<String, String>) -> Self {
    Self {
      nat: dict.get("NAT").map(String::to_string).unwrap_or_default(),
      ip: dict.get("IP").map(String::to_string).unwrap_or_default(),
      dns1: dict.get("DNS1").map(String::to_string).unwrap_or_default(),
      dns2: dict.get("DNS2").map(String::to_string).unwrap_or_default(),
      dns3: dict.get("DNS3").map(String::to_string).unwrap_or_default(),
      mac: dict.get("WAN MAC").map(String::to_string).unwrap_or_default(),
      gateway: dict.get("网关").map(String::to_string).unwrap_or_default(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode")]
pub enum WanInfo {
  /// 模式: PPPoE
  PPPoE {
    /// 连接名称: 3_INTERNET_R_VID_
    name: String,
    ip_info: WanIpInfo,
    /// 连接状态: 连接
    status: String,
    /// 断开原因:
    error_reason: String,
    /// 在线时长: 1156992秒
    uptime: String,
  },

  /// 模式: DHCP
  DHCP {
    /// 连接名称: 1_TR069_VOICE_R_VID_46
    name: String,
    ip_info: WanIpInfo,
    /// 连接状态: 连接
    status: String,
    /// 剩余租期: 1156992秒
    lease_time: String,
  },

  /// 模式: 桥接
  Bridge {
    /// 连接名称: 2_Other_B_VID_85
    name: String,
  },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortForwardingProtocol {
  #[serde(rename = "0")]
  TCP,
  #[serde(rename = "1")]
  UDP,
  #[serde(rename = "2")]
  Both,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortForwardingAction {
  New, Apply(u32), Delete(u32),
}


#[derive(Debug, Clone, PartialEq)]
pub enum PortForwardingPort {
  Simple(u32),
  Transform { remote: u32, local: u32 },
  Multiple { remote: (u32, u32), local: (u32, u32) },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortForwardingParam {
  // "0": true, "1": false
  #[serde(with = "serde_str01_as_bool")]
  pub enable: bool,
  pub name: String,
  // "0": TCP, "1": UDP, "2", TCP AND UDP
  pub protocol: PortForwardingProtocol,
  #[serde(rename="WANCViewName")]
  pub wan_interface: String,
  #[serde(rename="MinRemoteHost", with="serde_strnull_as_option")]
  pub remote_addr_min: Option<String>,
  #[serde(rename="MaxRemoteHost", with="serde_strnull_as_option")]
  pub remote_addr_max: Option<String>,
  #[serde(rename="MinExtPort", with="serde_str_as_u32")]
  pub remote_port_min: u32,
  #[serde(rename="MaxExtPort", with="serde_str_as_u32")]
  pub remote_port_max: u32,
  #[serde(rename="InternalHost", with="serde_strnull_as_option")]
  pub local_addr: Option<String>,
  #[serde(rename="InternalMacHost", with="serde_strnull_as_option")]
  pub local_mac: Option<String>,
  #[serde(rename="MacEnable", with="serde_str01_as_bool")]
  pub enable_local_mac: bool,
  #[serde(rename="MinIntPort", with="serde_str_as_u32")]
  pub local_port_min: u32,
  #[serde(rename="MaxIntPort", with="serde_str_as_u32")]
  pub local_port_max: u32,
  #[serde(with="serde_strnull_as_option")]
  pub description: Option<String>,
  #[serde(with="serde_strnull_as_option")]
  pub lease_duration: Option<String>,
  #[serde(rename="PortMappCreator", with="serde_strnull_as_option")]
  pub port_map_creator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortForwardingError {
  pub error_str: String,
  pub error_param: String,
  pub error_type: String,
}

pub struct Context {
  pub base_url: String,
  pub client: reqwest::Client,
  pub session: Option<Session>,
}

impl Context {
  pub fn new<S: ToString>(base_url: S) -> Self {
    Self {
      base_url: base_url.to_string(),
      client: reqwest::Client::new(),
      session: None,
    }
  }

  pub fn base_url(&self) -> &str { &self.base_url }
  pub fn template_url(&self) -> String { format!("{}/template.gch", self.base_url) }
  pub fn next_url(&self, page: &str) -> String {
    format!("{}/{}{}", self.base_url, self.session.as_ref().map(|s| s.url_next.as_str()).unwrap_or("getpage.gch?pid=1002&nextpage="), page)
  }

  fn update_session(&mut self, result: &str) -> Result<bool> {
    let session_token = match result.split("var session_token = ").nth(1) {
      Some(s) => s,
      _ => return Ok(false),
    }.splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("session_token parse failed"))?;
    let url_next = result
      .split("function getURL(){var ret = ").nth(1).ok_or_else(|| anyhow::format_err!("url_next not found"))?
      .splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("url_next parse failed"))?;
    debug!("session_token: {}", session_token);
    self.session = Some(Session { url_next: url_next.to_string(), session_token: session_token.to_string() });
    Ok(true)
  }

  pub async fn do_login(&mut self, username: &str, password: &str) -> Result<()> {
    let session = self.login(username, password).await?;
    self.session = Some(session);
    Ok(())
  }
  pub async fn login(&self, username: &str, password: &str) -> Result<Session> {
    #[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum LoginAction {
      Login,
    }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct LoginParam {
      freshnum: String,
      action: LoginAction,
      /// this is come from line
      /// `getObj("Frm_Logintoken").value = "4";`
      #[serde(rename = "Frm_Logintoken")]
      login_token: String,
      #[serde(rename = "Username")]
      username: String,
      #[serde(rename = "Password")]
      password: String,
    }
    let client = self.client.clone();
    let result = client.get(self.base_url()).send().await?.text().await?;
    let login_token = result
      .split(r#"getObj("Frm_Logintoken").value = "#).nth(1).unwrap_or(r#""1""#)
      .splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("login_token parse failed"))?;

    debug!("login_token: {}", login_token);
    let login_param = LoginParam {
      freshnum: "".to_string(),
      action: LoginAction::Login,
      login_token: login_token.to_string(),
      username: username.to_string(),
      password: password.to_string(),
    };

    debug!("{:?}", login_param);
    let result = client.post(self.base_url()).form(&login_param).send().await?.text().await?;
    if result.len() > 0 && result.find(r#"<iframe width="808px" height="67px" src="top.gch" name="topFrame" scrolling="no" frameborder="0" id="topFrame"></iframe>"#).is_none() {
      // parse error message
      // `getObj("errmsg").innerHTML = "用户信息有误，请重新输入。";`
      debug!("content length: {}", result.len());
      let errmsg = result
        .replace("function SetDisabled()\n{\ngetObj(\"errmsg\").innerHTML", "function SetDisabled()\n{\ngetObj(\"errmsg\") .innerHTML")
        .split(r#"getObj("errmsg").innerHTML = "#).nth(1).unwrap_or(r#""login might failed""#)
        .splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("errmsg parse failed"))?
        .to_string();
      error!("errmsg: {}", errmsg);
      anyhow::bail!("login failed");
    }

    let result = client.get(self.template_url()).send().await?.text().await?;
    let session_token = result
      .split("var session_token = ").nth(1).ok_or_else(|| anyhow::format_err!("session_token not found"))?
      .splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("session_token parse failed"))?;
    let url_next = result
      .split("function getURL(){var ret = ").nth(1).ok_or_else(|| anyhow::format_err!("url_next not found"))?
      .splitn(3, '"').nth(1).ok_or_else(|| anyhow::format_err!("url_next parse failed"))?;
    debug!("session_token: {}", session_token);
    Ok(Session {
      url_next: url_next.to_string(),
      session_token: session_token.to_string(),
    })
  }

  pub async fn wan_info(&self) -> Result<Vec<WanInfo>> {
    use select::predicate::{Class, Name, Predicate};
    let client = self.client.clone();
    let result = client.get(self.next_url("status_ethwan_if_t.gch")).send().await?.text().await?;
    let dom = select::document::Document::from_read(result.as_bytes())?;
    let mut result = Vec::new();
    for table in dom.find(Name("div").and(Class("space_0"))) {
      let mut kv = HashMap::new();
      for tr in table.find(Name("tr")) {
        let mut td = tr.find(Name("td")).map(|i| {
          // might be <Input type="text" class="uiNoBorder" style="text-align:left;" size=45 value="桥接" readonly>
          match i.first_child() {
            Some(e) => {
              if e.name() == Some("input") {
                e.attr("value").unwrap_or_default().to_string()
              } else {
                i.text()
              }
            },
            _ => i.text()
          }.trim().to_string()
        });
        kv.entry(td.next().unwrap_or_default()).or_insert(td.next().unwrap_or_default());
      }
      let wan_info = match kv.get("模式").map(String::as_str) {
        Some("PPPoE") => {
          let ip_info = WanIpInfo::from_dict(&kv);
          WanInfo::PPPoE {
            name: kv.get("连接名称").map(String::to_string).unwrap_or_default(),
            ip_info,
            status: kv.get("连接状态").map(String::to_string).unwrap_or_default(),
            error_reason: kv.get("断开原因").map(String::to_string).unwrap_or_default(),
            uptime: kv.get("在线时长").map(String::to_string).unwrap_or_default(),
          }
        },
        Some("DHCP") => {
          let ip_info = WanIpInfo::from_dict(&kv);
          WanInfo::DHCP {
            name: kv.get("连接名称").map(String::to_string).unwrap_or_default(),
            ip_info,
            status: kv.get("连接状态").map(String::to_string).unwrap_or_default(),
            lease_time: kv.get("剩余租期").map(String::to_string).unwrap_or_default(),
          }
        },
        Some("桥接") => {
          WanInfo::Bridge {
            name: kv.get("连接名称").map(String::to_string).unwrap_or_default(),
          }
        },
        _ => {
          error!("unknown wan_info: {:?}", kv);
          anyhow::bail!("unknown wan_info");
        }
      };
      result.push(wan_info);
    }
    Ok(result)
  }

  fn parse_forwarding_list(result: &str) -> Result<(PortForwardingError, Vec<PortForwardingParam>)> {
    fn parse_transfer_meaning(result: &str, field: &str) -> Option<String> {
      let value = result
        .split(&format!("Transfer_meaning('{}',", field)).skip(1).last()?
        .splitn(2, ')').nth(0)?
        .trim().strip_prefix('\'')?.strip_suffix('\'')?
        .replace("\\x2d", "\x2d")
        .replace("\\x2e", "\x2e")
        .replace("\\x3a", "\x3a")
        .replace("\\x5f", "\x5f")
        .to_string();
      Some(value)
    }
    let error_str = parse_transfer_meaning(result, "IF_ERRORSTR").unwrap_or_else(|| "SUCC".to_string());
    let error_param = parse_transfer_meaning(result, "IF_ERRORPARAM").unwrap_or_else(|| "SUCC".to_string());
    let error_type = parse_transfer_meaning(result, "IF_ERRORTYPE").unwrap_or_else(|| "SUCC".to_string());
    debug!("error_str: {}", error_str);
    debug!("error_param: {}", error_param);
    debug!("error_type: {}", error_type);

    let mut list = Vec::new();
    let count = parse_transfer_meaning(result, "IF_INSTNUM").unwrap_or_default()
      .parse::<usize>().map_err(|_| anyhow::format_err!("parse IF_INSTNUM"))?;
    for i in 0..count {
      list.push(PortForwardingParam {
        enable: parse_transfer_meaning(result, &format!("Enable{}", i)).unwrap_or_default() == "1",
        name: parse_transfer_meaning(result, &format!("Name{}", i)).unwrap_or_default(),
        protocol: match parse_transfer_meaning(result, &format!("Protocol{}", i)).unwrap_or_default().as_str() {
          "0" => PortForwardingProtocol::TCP,
          "1" => PortForwardingProtocol::UDP,
          "2" => PortForwardingProtocol::Both,
          _ => anyhow::bail!("unknown protocol"),
        },
        wan_interface: parse_transfer_meaning(result, &format!("WANCViewName{}", i)).unwrap_or_default(),
        remote_addr_min: parse_transfer_meaning(result, &format!("MinRemoteHost{}", i)),
        remote_addr_max: parse_transfer_meaning(result, &format!("MaxRemoteHost{}", i)),
        remote_port_min: parse_transfer_meaning(result, &format!("MinExtPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MinExtPort"))?,
        remote_port_max: parse_transfer_meaning(result, &format!("MaxExtPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MaxExtPort"))?,
        local_addr: parse_transfer_meaning(result, &format!("InternalHost{}", i)),
        local_mac: parse_transfer_meaning(result, &format!("InternalMacHost{}", i)),
        enable_local_mac: parse_transfer_meaning(result, &format!("MacEnable{}", i)).unwrap_or_default() == "1",
        local_port_min: parse_transfer_meaning(result, &format!("MinIntPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MinIntPort"))?,
        local_port_max: parse_transfer_meaning(result, &format!("MaxIntPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MaxIntPort"))?,
        description: parse_transfer_meaning(result, &format!("Description{}", i)),
        port_map_creator: parse_transfer_meaning(result, &format!("PortMappCreator{}", i)),
        lease_duration: parse_transfer_meaning(result, &format!("LeaseDuration{}", i)),
      });
    }

    if error_str != "SUCC" {
      error!("error: {}, {}, {}", error_str, error_param, error_type);
    }

    Ok((PortForwardingError {
      error_str: error_str.to_string(),
      error_param: error_param.to_string(),
      error_type: error_type.to_string(),
    }, list))
  }

  pub async fn port_forwarding_list(&mut self) -> Result<Vec<PortForwardingParam>> {
    let client = self.client.clone();
    let result = client.get(self.next_url("app_virtual_conf_t.gch")).send().await?
      .text().await?;
    self.update_session(&result)?;
    let (_, list) = Self::parse_forwarding_list(&result)?;
    Ok(list)
  }

  pub async fn port_forwarding(&mut self, action: PortForwardingAction, name: &str, protocol: PortForwardingProtocol, wan: &str, lan: &str, port: PortForwardingPort) -> Result<Vec<PortForwardingParam>> {
    let client = self.client.clone();

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum PortForwardingActionName {
      New, Apply, Delete,
    }
    // pub enum LocalHost {
    //   Host(String),
    //   MacHost(String),
    // }
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PortForwardingRequest {
      #[serde(rename="IF_ACTION")]
      action: PortForwardingActionName,
      #[serde(rename="IF_INDEX")]
      index: i32,
      #[serde(flatten)]
      params: PortForwardingParam,
      #[serde(rename="_SESSION_TOKEN")]
      _session_token: String,
    }
    let (remote_port_min, remote_port_max, local_port_min, local_port_max) = match port {
      PortForwardingPort::Simple(p) => (p, p, p, p),
      PortForwardingPort::Transform { remote, local } => (remote, remote, local, local),
      PortForwardingPort::Multiple { remote: (remote_min, remote_max), local: (local_min, local_max) } => (remote_min, remote_max, local_min, local_max),
    };
    let result = client.post(self.next_url("app_virtual_conf_t.gch")).form(&PortForwardingRequest {
      action: match action {
        PortForwardingAction::New => PortForwardingActionName::New,
        PortForwardingAction::Apply(_) => PortForwardingActionName::Apply,
        PortForwardingAction::Delete(_) => PortForwardingActionName::Delete,
      },
      index: match action {
        PortForwardingAction::New => -1,
        PortForwardingAction::Apply(i) => i as i32,
        PortForwardingAction::Delete(i) => i as i32,
      },
      params: PortForwardingParam {
        enable: true,
        name: name.to_string(),
        protocol,
        wan_interface: wan.to_string(),
        remote_addr_min: None,
        remote_addr_max: None,
        remote_port_min,
        remote_port_max,
        local_addr: Some(lan.to_string()),
        local_mac: None,
        enable_local_mac: false,
        local_port_min,
        local_port_max,
        description: None,
        port_map_creator: None,
        lease_duration: None,
      },
      _session_token: self.session.as_ref().map(|i| i.session_token.as_str()).unwrap_or_default().to_string(),
    }).send().await?.text().await?;
    self.update_session(result.as_str())?;
    let (err, list) = Self::parse_forwarding_list(&result)?;
    if err.error_str != "SUCC" {
      anyhow::bail!("port forwarding failed: {:?}", err);
    }
    // std::fs::write("cache.html", &result).ok();
    Ok(list)
  }
}

#[cfg(test)]
mod test {
  use super::*;

async fn get_ctx() -> Result<Context> {
  dotenvy::dotenv().ok();
  flexi_logger::Logger::try_with_env_or_str("info")?.start().ok();
  let username = std::env::var("router_username").unwrap_or_else(|_| "admin".to_string());
  let password = std::env::var("router_password").unwrap_or_else(|_| "password".to_string());
  info!("login as {}", username);
  let mut ctx = Context::new("http://192.168.1.1");
  ctx.do_login(&username, &password).await?;
  Ok(ctx)
}

#[tokio::test]
async fn test_login() -> Result<()> {
  let ctx = get_ctx().await?;
  info!("{:?}", ctx.session);
  let wan_info = ctx.wan_info().await?;
  info!("{:?}", wan_info);
  Ok(())
}

#[tokio::test]
async fn test_port_forwarding() -> Result<()> {
  let mut ctx = get_ctx().await?;

  for i in 0..10 {
    ctx.port_forwarding(
      PortForwardingAction::New,
      &format!("__test_rust_onu__{}", i),
      PortForwardingProtocol::TCP,
      "IGD.WD1.WCD3.WCPPP1",
      "1.1.1.1",
      PortForwardingPort::Simple(1050+i)).await?;
  }

  let list = ctx.port_forwarding_list().await?;
  debug!("{:?}", list);

  'outer: loop {
    let list = ctx.port_forwarding_list().await?;
    for (i, t) in list.iter().enumerate() {
      if t.name.starts_with("__test_rust_onu__") {
        info!("deleting {} {}", i, t.name);
        ctx.port_forwarding(PortForwardingAction::Delete(i as _), &t.name, PortForwardingProtocol::TCP, "", "", PortForwardingPort::Simple(0)).await?;
        continue 'outer;
      }
    }
    break;
  }
  Ok(())
}

#[test]
fn test_parse() -> Result<()> {
  let result = std::fs::read_to_string("cache.html")?;
  let (err, list) = Context::parse_forwarding_list(&result)?;
  println!("{:?}", err);
  println!("{:?}", list);
  Ok(())
}

}
