use std::env;

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
    let body_raw = reqwest::blocking::get(format!(
        "{}/v2/_catalog", 
        env::var("ADDR").unwrap_or("".to_string())
    ))
        .unwrap()
        .text()
        .unwrap();
    return serde_json::from_str(&body_raw).unwrap();
}

pub fn get_image(image: String) -> Image {
    let body_raw = reqwest::blocking::get(format!(
        "{}/v2/{}/tags/list", 
        env::var("ADDR").unwrap_or("".to_string()),
        image,
    ))
        .unwrap()
        .text()
        .unwrap();

    return serde_json::from_str(&body_raw).unwrap();
}
