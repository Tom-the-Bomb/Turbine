use std::collections::HashMap;

use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;

const URL: &'static str =
    "https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml";

fn default_color() -> String {
    "#FFFFFF".to_string()
}

#[derive(Deserialize, Debug)]
struct RawEntry {
    r#type: String,
    #[serde(default = "default_color")]
    color: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    extensions: Vec<String>,
    ace_mode: String,
}

#[derive(Serialize, Debug)]
struct Entry {
    r#type: String,
    name: String,
    color: String,
    aliases: Vec<String>,
    filenames: Vec<String>,
    extensions: Vec<String>,
    ace_mode: String,
}

async fn load_raw_yaml() -> Vec<u8> {
    let https = HttpsConnector::new();

    let response = Client::builder()
        .build::<_, hyper::Body>(https)
        .get(Uri::from_static(URL))
        .await
        .expect("http error");

    hyper::body::to_bytes(response)
        .await
        .expect("could not read bytes")
        .to_vec()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = match std::env::args().skip(1).next().map(|p| p.into()) {
        Some(p) => p,
        None => std::env::current_dir()?
            .parent()
            .expect("linguist should not be in the root directory")
            .join("frontend")
            .join("public"),
    };
    std::env::set_current_dir(output_dir)?;

    let data = serde_yaml::from_slice::<HashMap<String, RawEntry>>(&*load_raw_yaml().await)?;

    let mut resolved = HashMap::new();

    for (
        name,
        RawEntry {
            r#type,
            aliases,
            mut ace_mode,
            color,
            mut extensions,
            filenames,
        },
    ) in data
    {
        // Special case: remove extension .ts/.tsx to prioritize TypeScript/TSX language
        if name == "XML" {
            extensions.retain(|ext| ext != ".ts" && ext != ".tsx");
        }

        // Special case: use the TSX ace mode instead of the default JavaScript
        else if name == "TSX" {
            ace_mode = "tsx".to_string();
        }

        // Special case: use the Kotlin ace mode instead of text
        else if name == "Kotlin" {
            ace_mode = "kotlin".to_string();
        }

        // Remove anything that can give a false assumption to the user that
        // the language is supported, even though it really is just text
        if ace_mode == "text" && name != "Text" {
            continue;
        }

        resolved.insert(
            name.clone(),
            Entry {
                name,
                aliases,
                r#type,
                ace_mode,
                color,
                extensions,
                filenames,
            },
        );
    }

    serde_json::to_writer(
        tokio::fs::File::create("languages.json")
            .await?
            .into_std()
            .await,
        &resolved,
    )?;

    Ok(())
}
