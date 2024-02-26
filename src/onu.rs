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
    std::fs::write("cache.html", &result).ok();
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
}

#[tokio::test]
async fn test_login() -> Result<()> {
  dotenvy::dotenv().ok();
  flexi_logger::Logger::try_with_env_or_str("info")?.start().ok();
  let username = std::env::var("router_username").unwrap_or_else(|_| "admin".to_string());
  let password = std::env::var("router_password").unwrap_or_else(|_| "password".to_string());
  info!("login as {}", username);
  let mut ctx = Context::new("http://192.168.1.1");
  ctx.do_login(&username, &password).await?;
  info!("{:?}", ctx.session);
  let wan_info = ctx.wan_info().await?;
  info!("{:?}", wan_info);
  Ok(())
}
