use std::fs;
use reqwest::{blocking::Client, header::{AUTHORIZATION, USER_AGENT}};
use serde::{Deserialize, Serialize};
use crate::{config};

#[derive(Debug, Deserialize)]
struct WorkItem {
    id: i32,
    fields: Fields,
    #[serde(rename = "_links")]
    links: Links,
}

#[derive(Debug, Deserialize)]
struct Fields {
    #[serde(rename = "System.Title")]
    title: String,
    #[serde(rename = "Microsoft.VSTS.TCM.Steps")]
    steps: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Links {
    #[serde(rename = "html")]
    html: Href,
}

#[derive(Debug, Deserialize)]
struct Href {
    href: String,
}

#[derive(Debug, Deserialize)]
pub struct StepsRaw {
    #[serde(rename = "step", default)]
    pub steps: Vec<StepRaw>,
}

#[derive(Debug, Deserialize)]
pub struct StepRaw {
    #[serde(rename = "type", default)]
    pub step_type: String,
    #[serde(rename = "parameterizedString", default)]
    pub texts: Vec<ParameterizedString>,
}

#[derive(Debug, Deserialize)]
pub struct ParameterizedString {
    #[serde(rename = "$value", default)]
    pub value: String,
}


#[derive(Debug, Serialize)]
pub struct StepSimplified {
    pub step: String,
    pub expected: String,
}

use html_escape::decode_html_entities;
use regex::Regex;

fn strip_html(input: &str) -> String {
    let decoded = decode_html_entities(input);
    let tag_re = Regex::new(r"<[^>]*>").unwrap();
    tag_re.replace_all(&decoded, "").to_string()
}

pub fn get_tfs_work_item(
    work_item_id: u32,
) -> Result<ParsedWorkItem, Box<dyn std::error::Error>> {

    let cfg = config::get_config();


    let url = format!(
        "{}/{}/{}/_apis/wit/workitems/{}?api-version=6.0",
        cfg.settings.server_url.trim_end_matches('/'),
        cfg.settings.org,
        cfg.settings.project,
        work_item_id
    );
/* 
    let auth = format!("Basic {}", base64::encode(format!(":{}",&cfg.settings.pat_token)));

    let client = Client::new();
    let response = client
        .get(&url)
        .header(USER_AGENT, "rust-client")
        .header(AUTHORIZATION, auth)
        .send()?
        .text()?;
*/
    let src = fs::read_to_string("work_item.json")?;

    let item: WorkItem = serde_json::from_str(&src)?;


    let steps_raw = quick_xml::de::from_str::<StepsRaw>(&item.fields.steps.unwrap_or_default());

    let mut simplified_steps = vec![];
    
    match steps_raw {
        Ok(steps) => {
            for s in steps.steps {
                let step = s.texts.get(0).map(|t| t.value.clone()).unwrap_or_default();
                let expected = s.texts.get(1).map(|t| t.value.clone()).unwrap_or_default();
                simplified_steps.push(StepSimplified { step: strip_html(&step), expected: strip_html(&expected) });
            }
        }
        Err(e) => {
            // Du kan vælge at logge fejlen, eller ignorere/returnere fejl
            eprintln!("FEJL ved parsing af steps: {e:?}");
        }
    }


    let result = ParsedWorkItem {
        id: item.id,
        title: item.fields.title,
        web_link: item.links.html.href,
        steps: simplified_steps
    };
//serde_json::to_string_pretty(&result)?
    Ok(result)
}


#[derive(Debug, Serialize)]
pub struct ParsedWorkItem {
    pub id: i32,
    pub title: String,
    pub web_link: String,
    pub steps: Vec<StepSimplified>,
}
