use anyhow::{Context, Result, bail};
use clap::Parser;
use digest_auth::{AuthContext, HttpMethod};
use serde_json::json;
use std::time::Duration;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_negative_numbers = true)]
struct Args {
    /// Base URL of the Fronius inverter (e.g., <http://192.168.1.100>)
    #[arg(short, long, env = "FRONIUS_BASE_URL", value_parser = clap::value_parser!(Url))]
    base_url: Url,

    /// Password for the "service" user
    #[arg(short, long, env = "FRONIUS_SERVICE_PASSWORD", hide_env_values = true)]
    password: String,

    /// Power limit in Watts (negative to turn off)
    p: i32,
}

fn main() -> Result<()> {
    // Attempt to load environment variables from a .env file (ignoring errors if not found)
    dotenvy::dotenv().ok();

    let args = Args::parse();
    set_export_limit(&args.base_url, &args.password, args.p)
}

fn set_export_limit(base_url: &Url, password: &str, p: i32) -> Result<()> {
    let mut url = base_url.join("/config/exportlimit/")?;
    url.query_pairs_mut().append_pair("method", "save");

    // digest_auth requires the exact URI path and query sent in the HTTP request
    // but without the schema, host, port, and fragment.
    let target_path = &url[url::Position::BeforePath..url::Position::AfterQuery];

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

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .http_status_as_error(false)
        .build()
        .into();

    // First request to obtain the digest auth challenge
    let mut initial_response = agent
        .post(url.as_str())
        .send_json(&data)
        .context("Failed to send initial request")?;

    // If the response is not a 401 digest challenge, handle it directly.
    if initial_response.status() != 401 {
        let status = initial_response.status();
        let body = initial_response.body_mut().read_to_string()?;

        if !status.is_success() {
            bail!("Request failed with status {status}: {body}");
        }
        println!("{body}");
        return Ok(());
    }

    // Extract the digest auth challenge from the 401 response.
    let header_val = initial_response
        .headers()
        .get("X-WWW-Authenticate")
        .or_else(|| initial_response.headers().get("WWW-Authenticate"))
        .and_then(|h| h.to_str().ok())
        .map(std::borrow::ToOwned::to_owned)
        .context("401 Unauthorized but no (X-)WWW-Authenticate header found")?;

    // Generate a digest response and try again.
    let mut prompt =
        digest_auth::parse(&header_val).context("Failed to parse Digest challenge")?;

    // The Fronius API always uses the fixed username "service" for controlling PV power limits
    let mut context = AuthContext::new("service", password, target_path);
    context.method = HttpMethod::POST;

    let answer = prompt
        .respond(&context)
        .context("Failed to generate digest response")?;

    let mut response = agent
        .post(url.as_str())
        .header("Authorization", &answer.to_header_string())
        .send_json(&data)
        .context("Failed to send authenticated request")?;
    let status = response.status();
    let body = response.body_mut().read_to_string()?;

    if !status.is_success() {
        bail!("Request failed with status {status}: {body}");
    }
    println!("{body}");

    Ok(())
}
