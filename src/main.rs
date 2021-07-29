mod api;
use crate::api::Api;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    //Init console parser
    let matches = App::new("EUserv extend")
        .version("1.0")
        .author("CRoot")
        .about("auto to extend EUserv")
        .arg(
            Arg::new("email")
                .short('e')
                .required(true)
                .about("Set your email.")
                .takes_value(true),
        )
        .arg(
            Arg::new("password")
                .short('p')
                .required(true)
                .about("Set your password.")
                .takes_value(true),
        )
        .get_matches();

    //Call EUserv api to login,check and renew
    let eu_api = Api::new(
        matches.value_of("email").unwrap().to_string(),
        matches.value_of("password").unwrap().to_string(),
    );
    let rep_body = eu_api.login().await?;
    if let Some(server_info) = eu_api.check_server(rep_body.rep_body) {
        //println!("debug server status：{:?}", server_info);
        if server_info.status {
            println!("{} server is ok.", server_info.server_id);
        } else {
            println!("{} server start to extend.", server_info.server_id);
            if eu_api
                .renew(server_info.server_id.clone(), rep_body.sess_id)
                .await?
            {
                println!("{} server successfully extend.", server_info.server_id);
            } else {
                println!("{} server unsuccessfully extend.", server_info.server_id);
            }
        }
    } else {
        println!("Cannot find your vps from EUserv.");
    }
    //end
    println!("Done.");
    Ok(())
}
