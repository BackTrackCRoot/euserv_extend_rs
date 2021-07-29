use kuchiki::traits::TendrilSink;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct ServerInfo {
    pub status: bool,
    pub server_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct LoginRep {
    pub rep_body: String,
    pub sess_id: String,
}

pub struct Api {
    client: Client,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenValue {
    value: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenRep {
    token: TokenValue,
    rc: String,
    rs: String,
}

impl Api {
    pub fn new(email: String, password: String) -> Self {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.70";
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(user_agent)
            .build()
            .expect("Http Clinit init failed!");
        Api {
            client,
            email,
            password,
        }
    }
    pub async fn login(&self) -> Result<LoginRep, reqwest::Error> {
        //get session id
        let res = self
            .client
            .get("https://support.euserv.com/index.iphp")
            .send()
            .await?;
        let login_url = res.url().to_string();
        //Anti python
        self.client
            .get("https://support.euserv.com/pic/logo_small.png")
            .send()
            .await?;
        let document = kuchiki::parse_html().one(res.text().await?);
        let elm = document
            .select_first("[name=sess_id]")
            .expect("Can not find sessio id")
            .attributes
            .borrow()
            .clone();
        let sess_id = elm.get("value").unwrap();
        //do longin
        let login_data = [
            ("email", self.email.as_str()),
            ("password", self.password.as_str()),
            ("form_selected_language", "en"),
            ("Submit", "Login"),
            ("subaction", "login"),
            ("sess_id", sess_id),
        ];

        let res = self.client.post(login_url).form(&login_data).send().await?;
        //let b = res.text().await?;
        //println!("{:?}", res);
        let ret_data = res.text().await?;
        if None != ret_data.find("Confirm or change your customer data here") {
            panic!("Your customer data must be checked and confirmed by you.");
        }
        if None == ret_data.find("Hello") {
            panic!("User/password is error.");
        }
        //check login status
        Ok(LoginRep {
            rep_body: ret_data,
            sess_id: sess_id.to_string(),
        })
    }

    pub fn check_server(&self, rep_body: String) -> Option<ServerInfo> {
        //check login status
        let document = kuchiki::parse_html().one(rep_body);
        let mut server_info = ServerInfo::default();
        //#kc2_order_customer_orders_tab_content_1
        //#kc2_order_customer_orders_tab_content_1 .kc2_order_table.kc2_content_table tr
        for css_match in document
            .select("#kc2_order_customer_orders_tab_content_1")
            .expect("Can not find your server list.")
        {
            if let Ok(as_node) = css_match.as_node().select_first(".td-z1-sp1-kc") {
                let server_id = as_node.text_contents();
                if server_id.len() == 1 {
                    continue;
                }
                //println!("Server ID:{}", server_id);
                server_info.server_id = server_id;
            } else {
                //println!("Debug node:{:?}", css_match.as_node().select_first(".td-z1-sp1-kc"));
                return None;
            }
            if let Ok(as_node) = css_match
                .as_node()
                .select_first(".td-z1-sp2-kc .kc2_order_action_container")
            {
                if as_node
                    .text_contents()
                    .contains("Contract extension possible from")
                {
                    //println!("Server Status:{}", as_node.text_contents());
                    server_info.status = true;
                } else {
                    server_info.status = false;
                }
                return Some(server_info);
            } else {
                server_info.status = false;
                return Some(server_info);
            }
        }
        return None;
    }

    pub async fn renew(&self, server_id: String, sess_id: String) -> Result<bool, reqwest::Error> {
        let post_url = "https://support.euserv.com/index.iphp";
        let post_data = [
            ("Submit", "Extend contract"),
            ("sess_id", sess_id.as_str()),
            ("ord_no", server_id.as_str()),
            ("subaction", "choose_order"),
            ("choose_order_subaction", "show_contract_details"),
        ];
        self.client.post(post_url).form(&post_data).send().await?;
        let post_data = [
            ("sess_id", sess_id.as_str()),
            ("subaction", "kc2_security_password_get_token"),
            ("prefix", "kc2_customer_contract_details_extend_contract_"),
            ("password", self.password.as_str()),
        ];
        let rep_json = self
            .client
            .post(post_url)
            .form(&post_data)
            .send()
            .await?
            .json::<TokenRep>()
            .await?;
        //println!("{:?}", rep_json);
        if rep_json.rs.eq("success") {
            //println!("{:?}", rep_json.token.value);
            let post_data = [
                ("sess_id", sess_id.as_str()),
                ("ord_no", server_id.as_str()),
                (
                    "subaction",
                    "kc2_customer_contract_details_extend_contract_term",
                ),
                ("token", rep_json.token.value.as_str()),
            ];
            self.client.post(post_url).form(&post_data).send().await?;
            return Ok(true);
        }
        Ok(false)
    }
}
