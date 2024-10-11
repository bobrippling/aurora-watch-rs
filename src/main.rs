#![allow(dead_code)]

use reqwest::blocking::get;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct AuroraWatch {
    current: Entry,
    previous: Entry,
    station: String,
    updated: String,
}

#[derive(Debug, Deserialize)]
struct Entry {
    state: State,
}

#[derive(Debug, Deserialize)]
struct State {
    name: String,
    value: String,
    color: String,
    #[serde(rename = "$value")]
    description: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://aurorawatch.lancs.ac.uk/api/0.1/status.xml";
    let response = get(url)?.text()?;

    /*
    let response = r##"
    <aurorawatch>
      <current>
        <state name="yellow" value="50" color="#ffff00">Minor geomagnetic activity</state>
      </current>
      <previous>
        <state name="green" value="0" color="#33ff33">No significant activity</state>
      </previous>
      <station>AWN/SUM. Sumburgh Head, UK.</station>
      <updated>2024-10-11 19:45:32</updated>
    </aurorawatch>
    "##;
    */

    let watch: AuroraWatch = serde_xml_rs::from_str(&response)?;

    println!("{:#?}", watch);

    Ok(())
}
