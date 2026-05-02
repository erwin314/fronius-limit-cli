use anyhow::{bail, Context, Result};
use clap::Parser;
use digest_auth::{AuthContext, HttpMethod};
use serde_json::json;
use std::time::Duration;
use url::Url;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_negative_numbers = true)]
struct Args {
    /// Base URL of the Fronius inverter (e.g., http://192.168.1.100)
    #[arg(short, long, env = "FRONIUS_BASE_URL")]
    base_url: String,

    /// Password for the "service" user
    #[arg(short, long, env = "FRONIUS_SERVICE_PASSWORD")]
    password: String,

    /// Power limit in Watts (negative to turn off)
    p: i32,
}

fn main() -> Result<()> {
    // Attempt to load environment variables from a .env file (ignoring errors if not found)
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let p = args.p;

    let mut url = Url::parse(&args.base_url)?.join("/config/exportlimit/")?;
    url.query_pairs_mut().append_pair("method", "save");
    
    // digest_auth requires the exact URI path and query sent in the HTTP request
    let target_path = format!("{}?{}", url.path(), url.query().unwrap_or(""));

    let data = json!({
        "powerLimits": {
            "exportLimits": {
                "activePower": {
                    "hardLimit": { "enabled": false, "powerLimit": 0 },
                    "mode": if p >= 0 { "entireSystem" } else { "off" },
                    "softLimit": { "enabled": p >= 0, "powerLimit": p.max(0) },
                },
                "failSafeModeEnabled": false,
            },
            "visualization": {
                "exportLimits": {
                    "activePower": {
                        "displayModeHardLimit": "absolute",
                        "displayModeSoftLimit": "absolute",
                    }
                },
                "wattPeakReferenceValue": p.max(0),
            },
        }
    });

    let agent = ureq::builder()
        .timeout(Duration::from_secs(10))
        .build();

    let result = agent.post(url.as_str()).send_json(&data);

    let (is_unauthorized, authenticate_header) = match &result {
        Err(ureq::Error::Status(401, res)) => {
            let auth_header = res.header("X-WWW-Authenticate")
                .or_else(|| res.header("WWW-Authenticate"))
                .map(|s| s.to_string());
            (true, auth_header)
        }
        _ => (false, None),
    };

    if is_unauthorized {
        if let Some(header_val) = authenticate_header {
            let mut prompt = digest_auth::parse(&header_val)
                .context("Failed to parse Digest challenge")?;

            let mut context = AuthContext::new("service", &args.password, &target_path);
            context.method = HttpMethod::POST;

            let answer = prompt
                .respond(&context)
                .context("Failed to generate digest response")?;

            let response = agent
                .post(url.as_str())
                .set("Authorization", &answer.to_header_string())
                .send_json(&data)
                .context("Failed to send authenticated request")?;

            println!("{}", response.into_string()?);
        } else {
            bail!("401 Unauthorized but no (X-)WWW-Authenticate header found");
        }
    } else {
        let response = result.context("Failed to send initial request")?;
        println!("{}", response.into_string()?);
    }

    Ok(())
}
