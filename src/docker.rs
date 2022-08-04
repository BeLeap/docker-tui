use std::env;

use log::debug;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Catalog {
    pub repositories: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Image {
    pub name: String,
    pub tags: Vec<String>,
}

pub fn get_catalog() -> Catalog {
    let request_url = format!(
        "{}/v2/_catalog",
        env::var("ADDR").unwrap_or("".to_string())
    );
    debug!("Request URL: {}", request_url);

    let body_raw = reqwest::blocking::get(request_url)
    .unwrap()
    .text()
    .unwrap();
    debug!("Raw Response: {}", body_raw);

    serde_json::from_str(&body_raw).unwrap()
}

pub fn get_image(image: String) -> Image {
    let request_url = format!(
        "{}/v2/{}/tags/list",
        env::var("ADDR").unwrap_or("".to_string()),
        image,
    );
    debug!("Request URL: {}", request_url);

    let body_raw = reqwest::blocking::get(request_url)
    .unwrap()
    .text()
    .unwrap();
    debug!("Raw Response: {}", body_raw);

    serde_json::from_str(&body_raw).unwrap()
}
