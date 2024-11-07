#![allow(dead_code)]

use reqwest::blocking::get;
use serde::Deserialize;
use log::debug;
use thiserror::Error;

type Result = std::result::Result<(), Error>;

#[derive(Error, Debug)]
enum Error {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("deserialisation failed: {0}")]
    Serde(#[from] serde_xml_rs::Error),
    #[error("connect failed")]
    Connect,
}

fn main() -> Result {
    env_logger::init();

    //v1();
    v2_status()?;

    Ok(())
}

fn v1() -> Result {
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

    let watch: AuroraWatch = fetch_xml("http://aurorawatch.lancs.ac.uk/api/0.1/status.xml")?;

    println!("{:#?}", watch);

    Ok(())
}

fn v2_status() -> Result {
    /*
    <?xml version='1.0' encoding='UTF-8' standalone='yes'?>
    <!DOCTYPE current_status PUBLIC "-//AuroraWatch-API//DTD REST 0.2.5//EN" "http://aurorawatch-api.lancs.ac.uk/0.2.5/aurorawatch-api.dtd">
    <current_status api_version="0.2.5">
        <updated>
            <datetime>2024-10-11T20:15:31+0000</datetime>
        </updated>
        <site_status
        project_id="project:AWN" site_id="site:AWN:SUM" site_url="http://aurorawatch-api.lancs.ac.uk/0.2.5/project/awn/sum.xml" status_id="yellow"/>
    </current_status>
    */

    #[derive(Debug, Deserialize)]
    struct Status {
        api_version: String,
        updated: Updated,
        site_status: SiteStatus,
    }

    #[derive(Debug, Deserialize)]
    struct Updated {
        datetime: DateTime,
    }

    #[derive(Debug, Deserialize)]
    struct DateTime(String);

    #[derive(Debug, Deserialize)]
    struct SiteStatus {
        project_id: String,
        site_id: String,
        site_url: String,
        status_id: String, // green yellow amber red
    }

    let watch: Status = fetch_xml("http://aurorawatch-api.lancs.ac.uk/0.2/status/current-status.xml")?;

    debug!("{:#?}", watch);

    println!("status {}", watch.site_status.status_id);

    Ok(())
}

fn v2_activity() -> ! {
    /*
    http://aurorawatch-api.lancs.ac.uk/0.2/status/alerting-site-activity.xml
    curl: (56) Recv failure: Operation timed out
    */
    todo!()
}

fn v2_descriptions() -> ! {
    /*
    http://aurorawatch-api.lancs.ac.uk/0.2/status-descriptions.xml
    <?xml version='1.0' encoding='UTF-8' standalone='yes'?>
    <!DOCTYPE status_list PUBLIC "-//AuroraWatch-API//DTD REST 0.2.5//EN" "http://aurorawatch-api.lancs.ac.uk/0.2/aurorawatch-api.dtd">
    <status_list api_version="0.2.5">
        <status id="green">
            <color>#33ff33</color>
            <description lang="en">No significant activity</description>
            <meaning lang="en">Aurora is unlikely to be visible by eye or camera from anywhere in the UK.</meaning>
        </status>
        <status id="yellow">
            <color>#ffff00</color>
            <description lang="en">Minor geomagnetic activity</description>
            <meaning lang="en">Aurora may be visible by eye from Scotland and may be visible by camera from Scotland, northern England and Northern Ireland.</meaning>
        </status>
        <status id="amber">
            <color>#ff9900</color>
            <description lang="en">Amber alert: possible aurora</description>
            <meaning lang="en">Aurora is likely to be visible by eye from Scotland, northern England and Northern Ireland; possibly visible from elsewhere in the UK. Photographs of aurora are likely from anywhere in the UK.</meaning>
        </status>
        <status id="red">
            <color>#ff0000</color>
            <description lang="en">Red alert: aurora likely</description>
            <meaning lang="en">It is likely that aurora will be visible by eye and camera from anywhere in the UK.</meaning>
        </status>
    </status_list>
    */
    todo!()
}

fn fetch_xml<'de, T: Deserialize<'de>>(url: &str) -> std::result::Result<T, Error> {
    let response = get_text(url)
        .map_err(|e| {
            use std::error::Error;
            if let Some(src) = e.source() {
                if let Some(src) = src.downcast_ref::<hyper::Error>() {
                    if src.is_connect() || src.is_timeout() {
                        return crate::Error::Connect
                    }
                }
            }
            crate::Error::Request(e)
        })?;

    let watch: T = serde_xml_rs::from_str(&response)?;
    Ok(watch)
}

fn get_text(url: &str) -> std::result::Result<String, reqwest::Error> {
    get(url)?.text()
}
