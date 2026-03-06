use anyhow::{Context, Result};
use bb_core::api::ApiClient;
use bb_core::config::Config;

pub async fn run(
    endpoint: String,
    method: String,
    fields: Vec<String>,
    headers: Vec<String>,
    jq: Option<String>,
    raw: bool,
    input: Option<String>,
) -> Result<()> {
    let config = Config::load()?;
    let credentials = config.credentials()?;
    let provider = config.provider();
    let client = ApiClient::new(&credentials, &provider)?;

    // Build JSON body from --field/-F key=value pairs or --input JSON string
    let body: Option<serde_json::Value> = if let Some(raw_input) = input {
        Some(serde_json::from_str(&raw_input).context("--input is not valid JSON")?)
    } else if !fields.is_empty() {
        let mut map = serde_json::Map::new();
        for f in &fields {
            let (k, v) = f.split_once('=').with_context(|| {
                format!("invalid --field format '{f}' — expected key=value")
            })?;
            // Try to parse value as JSON (number, bool, null), fall back to string
            let val = serde_json::from_str::<serde_json::Value>(v)
                .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
            map.insert(k.to_string(), val);
        }
        Some(serde_json::Value::Object(map))
    } else {
        None
    };

    // Parse --header/-H "Key: Value" pairs
    let extra_headers: Vec<(String, String)> = headers
        .iter()
        .map(|h| {
            h.split_once(':')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                .with_context(|| format!("invalid --header format '{h}' — expected 'Key: Value'"))
        })
        .collect::<Result<Vec<_>>>()?;

    let (status, body_text) = client
        .request(&method, &endpoint, body.as_ref(), &extra_headers)
        .await?;

    if !status_ok(status) {
        eprintln!("bb: server returned {status}");
        eprintln!("{body_text}");
        std::process::exit(1);
    }

    // If --jq is set, pipe through jq
    if let Some(filter) = &jq {
        run_jq(&body_text, filter)?;
        return Ok(());
    }

    // If --raw, print as-is (useful for binary-ish or plain text endpoints)
    if raw {
        print!("{body_text}");
        return Ok(());
    }

    // Try to pretty-print as JSON, otherwise print raw
    match serde_json::from_str::<serde_json::Value>(&body_text) {
        Ok(json) => println!("{}", serde_json::to_string_pretty(&json)?),
        Err(_) => print!("{body_text}"),
    }

    Ok(())
}

fn status_ok(status: u16) -> bool {
    (200..300).contains(&status)
}

fn run_jq(input: &str, filter: &str) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("jq")
        .arg(filter)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to run jq — make sure jq is installed (brew install jq)")?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .context("failed to write to jq stdin")?;

    let status = child.wait()?;
    anyhow::ensure!(status.success(), "jq exited with error");
    Ok(())
}
