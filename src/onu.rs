use std::{collections::HashMap, path::{Path, PathBuf}};

use anyhow::Result;
use select::predicate::Predicate;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Session {
  /// parsing from line
  /// `function getURL(){var ret = "getpage.gch?pid=1002&nextpage=";`
  url_next: String,
  /// parsing from line
  /// `var session_token = "859208547885....";`
  session_token: String,
}

// this mod is a serde helper that parse json string "1" to rust true, "0" to rust false
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

/// this mod is a serde helper that parse json string "123" to rust u32 123
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

/// this mod is a serde helper that parse json string "NULL" to rust None
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

#[allow(unused)]
fn display_debug<T: std::fmt::Debug>(o: &T) -> String {
  format!("{:?}", o)
}
fn display_option<T: std::fmt::Display>(o: &Option<T>) -> String {
  match o {
    Some(s) => format!("{}", s),
    None => format!(""),
  }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
#[serde(tag = "mode")]
pub enum WanInfo {
  /// 模式: PPPoE
  PPPoE {
    /// 连接名称: 3_INTERNET_R_VID_
    name: String,
    #[serde(flatten)]
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
    #[serde(flatten)]
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


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
pub struct LanInfo {
  /// 客户端名称/主机名
  /// HostName
  pub name: String,
  /// MAC地址: xx:xx:xx:xx:xx:xx
  /// MACAddr
  pub mac: String,
  /// IP地址: 0.0.0.0
  /// IPAddr
  pub ip: String,
  /// 剩余租期: 58473
  /// ExpiredTime
  pub lease_time: String,
  /// 端口: LAN4
  /// PhyPortName
  pub interface: String,
}


/// parsing from
/// ```html
/// <select name="Frm_WANCViewName" id="Frm_WANCViewName"  size="1" class="list_13">
/// <option value="IGD.WD1.WCD3.WCPPP1" ipmode="3" >&#51;&#95;&#73;&#78;&#84;&#69;&#82;&#78;&#69;&#84;&#95;&#82;&#95;&#86;&#73;&#68;&#95;</option>
/// </select>
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
pub struct WancInfo {
  /// Transfer_meaning('WANCName', '')
  /// <option value>{ name }</option>
  pub name: String,
  /// Transfer_meaning('WANCViewName', '')
  /// <option value="IGD.WD1.WCD3.WCPPP1">
  pub view_name: String,
  /// Transfer_meaning('ViewName', '')
  pub desc_name: String,
  /// <option ipmode="3">
  pub ipmode: isize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
pub struct Wan6Info {
  /// PPP
  pub name: String,
  /// GUA获取方式: SLAAC
  pub gua_assignment: String,
  /// GUA
  pub gua: String,
  /// DNS获取方式: DHCPv6
  pub dns_assignment: String,
  /// DNS1
  pub dns1: String,
  /// DNS1
  pub dns2: String,
  /// DNS1
  pub dns3: String,
  /// WAN MAC
  pub wan_mac: String,
  /// 前缀获取方式: DHCPv6
  pub prefix_assignment: String,
  /// 获取前缀
  pub prefix: String,
  /// 网关获取方式
  pub gateway_assignment: String,
  /// 网关
  pub gateway: String,
  /// 连接状态
  pub status: String,
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
impl std::fmt::Display for PortForwardingProtocol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PortForwardingProtocol::TCP => write!(f, "TCP"),
      PortForwardingProtocol::UDP => write!(f, "UDP"),
      PortForwardingProtocol::Both => write!(f, "TCP/UDP"),
    }
  }
}
impl std::str::FromStr for PortForwardingProtocol {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let value = match s.to_lowercase().as_str() {
      "tcp" => Self::TCP,
      "udp" => Self::UDP,
      "both" | "tcp/udp" | "udp/tcp" => Self::Both,
      _ => return Err("unknown protocol"),
    };
    Ok(value)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortForwardingAction {
  New, Apply(u32),
  Delete(u32), DeleteByName(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortForwardingHost {
  /// InternalHost, with MacEnable=false
  Host(String),
  /// InternalMacHost, with MacEnable=true
  Mac(String),
}
impl std::fmt::Display for PortForwardingHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PortForwardingHost::Host(s) => write!(f, "{}", s),
      PortForwardingHost::Mac(s) => write!(f, "{}", s),
    }
  }
}
impl std::str::FromStr for PortForwardingHost {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains(':') {
      Ok(PortForwardingHost::Mac(s.to_string()))
    } else {
      Ok(PortForwardingHost::Host(s.to_string()))
    }
  }
}

impl PortForwardingHost {
  pub fn as_host(&self) -> Option<&str> {
    match self {
      PortForwardingHost::Host(s) => Some(s.as_str()),
      _ => None,
    }
  }
  pub fn as_mac(&self) -> Option<&str> {
    match self {
      PortForwardingHost::Mac(s) => Some(s.as_str()),
      _ => None,
    }
  }
  pub fn is_mac(&self) -> bool {
    match self {
      PortForwardingHost::Mac(_) => true,
      _ => false,
    }
  }
}


#[derive(Debug, Clone, PartialEq)]
pub enum PortForwardingPort {
  /// MinIntPort = MaxIntPort = MinExtPort = MaxExtPort
  Simple(u32),
  /// MinIntPort = MaxIntPort = local
  /// MinExtPort = MaxExtPort = remote
  Transform { remote: u32, local: u32 },
  /// MinIntPort = local.0, MaxIntPort = local.1
  /// MinExtPort = remote.0, MaxExtPort = remote.1
  Multiple { remote: (u32, u32), local: (u32, u32) },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, tabled::Tabled)]
pub struct PortForwardingParam {
  /// "0": true, "1": false
  #[serde(with = "serde_str01_as_bool")]
  pub enable: bool,
  pub name: String,
  /// "0": TCP, "1": UDP, "2", TCP AND UDP
  pub protocol: PortForwardingProtocol,
  /// "IGD.WD1.***", wan interface view name (maybe vlan related)
  #[serde(rename="WANCViewName")]
  pub wan_interface: String,
  /// allowed remote host: `remote_addr_min..=remote_addr_max`
  #[serde(rename="MinRemoteHost", with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub remote_addr_min: Option<String>,
  /// allowed remote host: `remote_addr_min..=remote_addr_max`
  #[serde(rename="MaxRemoteHost", with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub remote_addr_max: Option<String>,
  /// listening port on router: `remote_port_min..=remote_port_max`
  #[serde(rename="MinExtPort", with="serde_str_as_u32")]
  pub remote_port_min: u32,
  /// listening port on router: `remote_port_min..=remote_port_max`
  #[serde(rename="MaxExtPort", with="serde_str_as_u32")]
  pub remote_port_max: u32,
  /// redirect to `local_addr` (ip address)
  #[serde(rename="InternalHost", with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub local_addr: Option<String>,
  /// redirect to `local_mac` (mac address)
  #[serde(rename="InternalMacHost", with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub local_mac: Option<String>,
  /// when enabled, `local_mac` is used, otherwise `local_addr` is used
  #[serde(rename="MacEnable", with="serde_str01_as_bool")]
  pub enable_local_mac: bool,
  /// redirect to `local_port_min..=local_port_max`
  #[serde(rename="MinIntPort", with="serde_str_as_u32")]
  pub local_port_min: u32,
  /// redirect to `local_port_min..=local_port_max`
  #[serde(rename="MaxIntPort", with="serde_str_as_u32")]
  pub local_port_max: u32,
  #[serde(with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub description: Option<String>,
  #[serde(with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub lease_duration: Option<String>,
  #[serde(rename="PortMappCreator", with="serde_strnull_as_option")]
  #[tabled(display_with = "display_option")]
  pub port_map_creator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiResult {
  /// 'SUCC' for success, "" might be not presented
  pub error_str: String,
  pub error_param: String,
  pub error_type: String,
}
impl ApiResult {
  pub fn is_success(&self) -> bool {
    if self.error_str == "" {
      warn!("maybe error: {:?}", self);
      return true
    }
    self.error_str == "SUCC"
  }
}

/// parsing from `<div>{text}</div>` or `<div><input type="text" value="{text}"></div>`
fn parse_node_text(node: select::node::Node<'_>) -> String {
  match node.children().find(|n| n.name()==Some("input")) {
    Some(e) => e.attr("value").unwrap_or_default().to_string(),
    None => node.text()
  }
}

/// parsing from `Transfer_meaning('{field}', '{value}');`
fn parse_transfer_meaning(resp: &str, field: &str) -> Option<String> {
  let value = resp
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

pub struct Request<'a> {
  session: &'a mut Option<Session>,
  cache_path: Option<&'a Path>,
  request: reqwest::RequestBuilder,
}

impl<'a> Request<'a> {
  fn parse_session(resp: &str) -> Option<Session> {
    let session_token = resp.split("var session_token = ").skip(1).last()?
      .splitn(3, '"').nth(1)?;
    let url_next = resp
      .split("function getURL(){var ret = ").nth(1)?
      .splitn(3, '"').nth(1)?;
    debug!("session_token: {}", session_token);
    Some(Session {
      url_next: url_next.to_string(),
      session_token: session_token.to_string(),
    })
  }

  pub async fn send(self) -> Result<(ApiResult, String)> {
    let (client, request) = self.request.build_split();
    let request = request?;
    debug!("request: {:?} {:?}", request.method(), request.url().as_str());
    let resp = client.execute(request).await?;
    let url = resp.url().to_string();
    let text = resp.text().await?;
    if let Some(cache_path) = self.cache_path {
      std::fs::write(&cache_path, &text)?;
      debug!("cache: {} => {}", url, cache_path.display());
    }
    if let Some(session) = Self::parse_session(&text) {
      debug!("update session: {}", session.session_token);
      *self.session = Some(session);
    }
    let err = Self::parse_api_result(&text);
    if !err.is_success() {
      error!("request {url} failed: {err:?}");
    }
    Ok((err, text))
  }

  pub fn form<T: serde::Serialize>(mut self, data: T) -> Self {
    #[derive(serde::Serialize)]
    struct WithSessionToken<T: serde::Serialize> {
      #[serde(flatten)]
      data: T,
      #[serde(rename = "_SESSION_TOKEN")]
      session_token: String,
    }
    self.request = match self.session {
      Some(session) => {
        let data = WithSessionToken {
          data,
          session_token: std::mem::take(&mut session.session_token),
        };
        session.session_token = String::new();
        self.request.form(&data)
      },
      None => {
        self.request.form(&data)
      },
    };
    self
  }

  fn parse_api_result(resp: &str) -> ApiResult {
    let error_str = parse_transfer_meaning(resp, "IF_ERRORSTR").unwrap_or_default();
    let error_param = parse_transfer_meaning(resp, "IF_ERRORPARAM").unwrap_or_default();
    let error_type = parse_transfer_meaning(resp, "IF_ERRORTYPE").unwrap_or_default();
    ApiResult {
      error_str,
      error_param,
      error_type,
    }
  }
}

pub struct Context {
  /// the ip of router, might be "http://192.168.1.1"
  pub base_url: String,
  /// if provided, last response would be saved to `cache_path`
  pub cache_path: Option<PathBuf>,
  /// internal `reqwest::Client`, use `Context::get` and `Context::set`
  /// for auto detect `ApiResult` and `update_session`
  pub _client: reqwest::Client,
  /// saved session_token, the session_token would be changed even in GET method
  pub session: Option<Session>,
}

impl Context {
  pub fn new<S: ToString>(base_url: S) -> Self {
    Self {
      base_url: base_url.to_string(),
      _client: reqwest::Client::new(),
      session: None,
      cache_path: None,
    }
  }

  pub fn base_url(&self) -> &str { &self.base_url }
  pub fn template_url(&self) -> String { format!("{}/template.gch", self.base_url) }
  pub fn next_url(&self, page: &str) -> String {
    format!("{}/{}{}", self.base_url, self.session.as_ref().map(|s| s.url_next.as_str()).unwrap_or("getpage.gch?pid=1002&nextpage="), page)
  }

  pub fn get(&mut self, page: &str) -> Request {
    let url = self.next_url(page);
    Request {
      session: &mut self.session,
      cache_path: self.cache_path.as_deref(),
      request: self._client.get(url),
    }
  }

  pub fn post(&mut self, page: &str) -> Request {
    let url = self.next_url(page);
    Request {
      session: &mut self.session,
      cache_path: self.cache_path.as_deref(),
      request: self._client.post(url),
    }
  }

  pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
    let session = self.init_session(username, password).await?;
    self.session = Some(session);
    Ok(())
  }

  pub async fn init_session(&self, username: &str, password: &str) -> Result<Session> {
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
    let client = self._client.clone();
    let resp = client.get(self.base_url()).send().await?.text().await?;
    let login_token = resp
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
    let resp = client.post(self.base_url()).form(&login_param).send().await?.text().await?;
    if resp.len() > 0 && resp.find(r#"<iframe width="808px" height="67px" src="top.gch" name="topFrame" scrolling="no" frameborder="0" id="topFrame"></iframe>"#).is_none() {
      // parse error message
      // `getObj("errmsg").innerHTML = "用户信息有误，请重新输入。";`
      debug!("content length: {}", resp.len());
      let errmsg = resp
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

  pub async fn wan_info(&mut self) -> Result<Vec<WanInfo>> {
    use select::predicate::{Class, Name};
    let (_, resp) = self.get("status_ethwan_if_t.gch").send().await?;
    let dom = select::document::Document::from_read(resp.as_bytes())?;
    let mut result = Vec::new();
    for table in dom.find(Name("div").and(Class("space_0"))) {
      // parse kv from
      // ```html
      // <div class="space_0">
      // <table id="TestContent" class="infor" width="410" border="0" cellpadding="0" cellspacing="1" bgcolor="#979797">
      // <tr class="white_1">
      // <td class="tdleft_1">模式</td>
      // <td class="tdright">PPPoE</td>
      // </tr>
      // ...
      // </table>
      // </div>
      // ```
      let mut kv = HashMap::new();
      for tr in table.find(Name("tr")) {
        let mut td = tr.find(Name("td")).map(|i| parse_node_text(i).trim().to_string());
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

  pub async fn lan_info(&mut self) -> Result<Vec<LanInfo>> {
    let (_, resp) = self.get("status_ethlan_dhcp_info_t.gch").send().await?;
    let count = parse_transfer_meaning(resp.as_str(), "IF_INSTNUM").unwrap_or_default()
      .parse::<usize>().map_err(|_| anyhow::format_err!("parse IF_INSTNUM"))?;
    let mut result = Vec::new();
    for i in 0..count {
      result.push(LanInfo {
        name: parse_transfer_meaning(resp.as_str(), &format!("HostName{}", i)).unwrap_or_default(),
        mac: parse_transfer_meaning(resp.as_str(), &format!("MACAddr{}", i)).unwrap_or_default(),
        ip: parse_transfer_meaning(resp.as_str(), &format!("IPAddr{}", i)).unwrap_or_default(),
        lease_time: parse_transfer_meaning(resp.as_str(), &format!("ExpiredTime{}", i)).unwrap_or_default(),
        interface: parse_transfer_meaning(resp.as_str(), &format!("PhyPortName{}", i)).unwrap_or_default(),
      });
    }
    Ok(result)
  }

  fn parse_forwarding_list(resp: &str) -> Result<Vec<PortForwardingParam>> {
    let mut list = Vec::new();
    let count = parse_transfer_meaning(resp, "IF_INSTNUM").unwrap_or_default()
      .parse::<usize>().map_err(|_| anyhow::format_err!("parse IF_INSTNUM"))?;
    for i in 0..count {
      list.push(PortForwardingParam {
        enable: parse_transfer_meaning(resp, &format!("Enable{}", i)).unwrap_or_default() == "1",
        name: parse_transfer_meaning(resp, &format!("Name{}", i)).unwrap_or_default(),
        protocol: match parse_transfer_meaning(resp, &format!("Protocol{}", i)).unwrap_or_default().as_str() {
          "0" => PortForwardingProtocol::TCP,
          "1" => PortForwardingProtocol::UDP,
          "2" => PortForwardingProtocol::Both,
          _ => anyhow::bail!("unknown protocol"),
        },
        wan_interface: parse_transfer_meaning(resp, &format!("WANCViewName{}", i)).unwrap_or_default(),
        remote_addr_min: parse_transfer_meaning(resp, &format!("MinRemoteHost{}", i)),
        remote_addr_max: parse_transfer_meaning(resp, &format!("MaxRemoteHost{}", i)),
        remote_port_min: parse_transfer_meaning(resp, &format!("MinExtPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MinExtPort"))?,
        remote_port_max: parse_transfer_meaning(resp, &format!("MaxExtPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MaxExtPort"))?,
        local_addr: parse_transfer_meaning(resp, &format!("InternalHost{}", i)),
        local_mac: parse_transfer_meaning(resp, &format!("InternalMacHost{}", i)),
        enable_local_mac: parse_transfer_meaning(resp, &format!("MacEnable{}", i)).unwrap_or_default() == "1",
        local_port_min: parse_transfer_meaning(resp, &format!("MinIntPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MinIntPort"))?,
        local_port_max: parse_transfer_meaning(resp, &format!("MaxIntPort{}", i)).unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse MaxIntPort"))?,
        description: parse_transfer_meaning(resp, &format!("Description{}", i)),
        port_map_creator: parse_transfer_meaning(resp, &format!("PortMappCreator{}", i)),
        lease_duration: parse_transfer_meaning(resp, &format!("LeaseDuration{}", i)),
      });
    }

    Ok(list)
  }

  pub fn parse_wanc_info(resp: &str) -> Result<Vec<WancInfo>> {
    use select::predicate::{Attr, Name};
    let mut result = select::document::Document::from_read(resp.as_bytes())?
      .find(Attr("id", "Frm_WANCViewName").descendant(Name("option"))).map(|option| {
        let name = option.text();
        let view_name = option.attr("value").unwrap_or_default().to_string();
        let ipmode = option.attr("ipmode").unwrap_or_default().parse().map_err(|_| anyhow::format_err!("parse ipmode"))?;
        anyhow::Ok(WancInfo {
          name: name.to_string(),
          view_name: view_name.to_string(),
          desc_name: String::new(),
          ipmode,
        })
      }).collect::<Result<Vec<_>,_>>()?;

    let count = parse_transfer_meaning(resp, "IF_INSTNUM").unwrap_or_default()
      .parse::<usize>().unwrap_or_default();
    for i in 0..count {
      let name = parse_transfer_meaning(resp, &format!("WANCName{}", i)).unwrap_or_default();
      let view_name = parse_transfer_meaning(resp, &format!("WANCViewName{}", i)).unwrap_or_default();
      let desc_name = parse_transfer_meaning(resp, &format!("ViewName{}", i)).unwrap_or_default();
      let found = result.iter_mut().find(|info| info.view_name == view_name).map(|i| {
        if i.desc_name.is_empty() {
          i.desc_name = desc_name.clone()
        }
      }).is_some();
      if !found {
        result.push(WancInfo {
          name,
          view_name,
          desc_name,
          ipmode: -1,
        });
      }
    }
    Ok(result)
  }

  pub async fn wanc_info(&mut self) -> Result<Vec<WancInfo>> {
    let (_, resp) = self.get("app_virtual_conf_t.gch").send().await?;
    let info = Self::parse_wanc_info(&resp)?;
    Ok(info)
  }

  pub async fn wan6_info(&mut self) -> Result<Vec<Wan6Info>> {
    use select::predicate::{Class, Name};
    let (_, resp) = self.get("status_wanstatu_ipv6wansta_t.gch").send().await?;
    let dom = select::document::Document::from_read(resp.as_bytes())?;

    let mut result = Vec::new();
    for table in dom.find(Name("div").and(Class("space_0"))) {
      let mut kv = HashMap::new();
      for tr in table.find(Name("tr")) {
        let mut td = tr.find(Name("td")).map(|i| parse_node_text(i).trim().to_string());
        kv.entry(td.next().unwrap_or_default()).or_insert(td.next().unwrap_or_default());
      }
      let info = Wan6Info {
        name: kv.get("PPP").map(String::to_string).unwrap_or_default(),
        gua_assignment: kv.get("GUA获取方式").map(String::to_string).unwrap_or_default(),
        gua: kv.get("GUA").map(String::to_string).unwrap_or_default(),
        dns_assignment: kv.get("DNS获取方式").map(String::to_string).unwrap_or_default(),
        dns1: kv.get("DNS1").map(String::to_string).unwrap_or_default(),
        dns2: kv.get("DNS2").map(String::to_string).unwrap_or_default(),
        dns3: kv.get("DNS3").map(String::to_string).unwrap_or_default(),
        wan_mac: kv.get("WAN MAC").map(String::to_string).unwrap_or_default(),
        prefix_assignment: kv.get("前缀获取方式").map(String::to_string).unwrap_or_default(),
        prefix: kv.get("获取前缀").map(String::to_string).unwrap_or_default(),
        gateway_assignment: kv.get("网关获取方式").map(String::to_string).unwrap_or_default(),
        gateway: kv.get("网关").map(String::to_string).unwrap_or_default(),
        status: kv.get("连接状态").map(String::to_string).unwrap_or_default(),
      };
      result.push(info)
    }
    Ok(result)
  }

  pub async fn port_forwarding_list(&mut self) -> Result<Vec<PortForwardingParam>> {
    let (_, resp) = self.get("app_virtual_conf_t.gch").send().await?;
    let list = Self::parse_forwarding_list(&resp)?;
    Ok(list)
  }

  /// only `PortForwardingAction::Delete` and `PortForwardingAction::DeleteByName` accepted,
  /// for other usage, see `Context::port_forwarding`
  pub async fn port_forwarding_delete(&mut self, action: PortForwardingAction) -> Result<Vec<PortForwardingParam>> {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PortForwardingRequest {
      #[serde(rename="IF_ACTION")]
      action: &'static str,
      #[serde(rename="IF_INDEX")]
      index: i32,
      #[serde(flatten)]
      params: PortForwardingParam,
    }
    let index = match action {
      PortForwardingAction::New | PortForwardingAction::Apply(_) =>
        anyhow::bail!("use port_forwarding instead"),
      PortForwardingAction::Delete(i) => i,
      PortForwardingAction::DeleteByName(name) => {
        let list = self.port_forwarding_list().await?;
        match list.iter().enumerate().find(|(_, p)| p.name == name) {
          Some((i, _)) => i as _,
          None => anyhow::bail!("cannot find name in list"),
        }
      },
    };
    let param = PortForwardingParam {
      enable: false,
      name: "".to_string(),
      protocol: PortForwardingProtocol::TCP,
      wan_interface: "".to_string(),
      remote_addr_min: None,
      remote_addr_max: None,
      remote_port_min: 0,
      remote_port_max: 0,
      local_addr: None,
      local_mac: None,
      enable_local_mac: false,
      local_port_min: 0,
      local_port_max: 0,
      description: None,
      port_map_creator: None,
      lease_duration: None,
    };
    let (err, resp) = self.post("app_virtual_conf_t.gch").form(&PortForwardingRequest {
      action: "delete",
      index: index as _,
      params: param,
    }).send().await?;
    if !err.is_success() {
      anyhow::bail!("port forwarding delete failed: {:?}", err);
    }
    let list = Self::parse_forwarding_list(&resp)?;
    Ok(list)
  }

  /// only `PortForwardingAction::New` and `PortForwardingAction::Apply` accepted,
  /// for other usage, see `Context::port_forwarding_delete`
  pub async fn port_forwarding(&mut self, action: PortForwardingAction, name: &str, protocol: PortForwardingProtocol, wan: &str, lan: PortForwardingHost, port: PortForwardingPort) -> Result<Vec<PortForwardingParam>> {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PortForwardingRequest {
      #[serde(rename="IF_ACTION")]
      action: &'static str,
      #[serde(rename="IF_INDEX")]
      index: i32,
      #[serde(flatten)]
      params: PortForwardingParam,
    }
    let (remote_port_min, remote_port_max, local_port_min, local_port_max) = match port {
      PortForwardingPort::Simple(p) => (p, p, p, p),
      PortForwardingPort::Transform { remote, local } => (remote, remote, local, local),
      PortForwardingPort::Multiple { remote: (remote_min, remote_max), local: (local_min, local_max) } => (remote_min, remote_max, local_min, local_max),
    };
    let (err, resp) = self.post("app_virtual_conf_t.gch").form(&PortForwardingRequest {
      action: match action {
        PortForwardingAction::New => "new",
        PortForwardingAction::Apply(_) => "apply",
        PortForwardingAction::Delete(_) | PortForwardingAction::DeleteByName(_) =>
          anyhow::bail!("use port_forwarding_delete instead"),
      },
      index: match action {
        PortForwardingAction::Apply(i) => i as i32,
        PortForwardingAction::New => -1,
        _ => unreachable!(),
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
        local_addr: lan.as_host().map(ToString::to_string),
        local_mac: lan.as_mac().map(ToString::to_string),
        enable_local_mac: lan.is_mac(),
        local_port_min,
        local_port_max,
        description: None,
        port_map_creator: None,
        lease_duration: None,
      },
    }).send().await?;
    if !err.is_success() {
      anyhow::bail!("port forwarding failed: {:?}", err);
    }
    let list = Self::parse_forwarding_list(&resp)?;
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
  ctx.login(&username, &password).await?;
  Ok(ctx)
}

#[tokio::test]
async fn test_login() -> Result<()> {
  let mut ctx = get_ctx().await?;
  info!("{:?}", ctx.session);
  let wan_info = ctx.wan_info().await?;
  info!("{:?}", wan_info);
  Ok(())
}

#[tokio::test]
async fn test_info() -> Result<()> {
  let mut ctx = get_ctx().await?;
  let wan_info = ctx.wan_info().await?;
  info!("{:?}", wan_info);
  let lan_info = ctx.lan_info().await?;
  info!("{:?}", lan_info);
  Ok(())
}

#[tokio::test]
async fn test_port_forwarding() -> Result<()> {
  let mut ctx = get_ctx().await?;

  async fn clean_up(ctx: &mut Context) -> Result<()> {
    loop {
      let list = ctx.port_forwarding_list().await?;
      for (i, t) in list.iter().enumerate().rev() {
        if t.name.starts_with("__test_rust_onu__") {
          info!("deleting {} {}", i, t.name);
          ctx.port_forwarding_delete(PortForwardingAction::Delete(i as _)).await?;
          continue;
        }
      }
      return Ok(())
    }
  }

  clean_up(&mut ctx).await?;
  for i in 0..10 {
    info!("adding {}", 1050+i);
    ctx.port_forwarding(
      PortForwardingAction::New,
      &format!("__test_rust_onu__{}", i),
      PortForwardingProtocol::TCP,
      "IGD.WD1.WCD3.WCPPP1",
      PortForwardingHost::Host("1.1.1.1".to_string()),
      PortForwardingPort::Simple(1050+i)).await?;
  }

  ctx.cache_path = Some("cache.html".into());
  let list = ctx.port_forwarding_list().await?;
  ctx.cache_path = None;
  debug!("{:?}", list);

  clean_up(&mut ctx).await?;
  Ok(())
}

#[test]
fn test_parse() -> Result<()> {
  let result = std::fs::read_to_string("cache.html")?;
  let err = Request::parse_api_result(&result);
  let list = Context::parse_forwarding_list(&result)?;
  println!("{:?}", err);
  println!("{:?}", list);
  Ok(())
}

#[tokio::test]
async fn test_parse_wan6() -> Result<()> {
  let mut ctx = get_ctx().await?;
  let _info = ctx.wan6_info().await?;
  Ok(())
}

}
