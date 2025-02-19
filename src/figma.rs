use reqwest::blocking::Client;
use serde_json::{Map, Value};

use crate::config::FigmaConfig;

pub struct FigmaImageExtractor;

impl FigmaImageExtractor {
  pub fn fetch_figma_images() -> Result<Option<Map<String, Value>>, reqwest::Error> {
    let FigmaConfig {
      figma_access_token,
      figma_api_url,
      figma_file_key,
      ..
    } = FigmaConfig::new();

    let client = Client::new();
    let file_url = format!("{}/images/{}", figma_api_url, figma_file_key);

    let response = client
      .get(&file_url)
      .query(&[
        ("format", "png"),
        ("ids", &Self::get_image_node_ids().join(",")),
      ])
      .header("X-Figma-Token", figma_access_token)
      .send()?
      .text()?;

    let response: Value = serde_json::from_str(&response).unwrap();
    let images = response["images"].as_object();

    Ok(images.cloned())
  }

  fn get_image_node_ids() -> Vec<String> {
    let FigmaConfig {
      figma_access_token,
      figma_api_url,
      figma_file_key,
      ..
    } = FigmaConfig::new();

    let client = Client::new();
    let file_url = format!("{}/files/{}", figma_api_url, figma_file_key);

    let response = client
      .get(&file_url)
      .header("X-Figma-Token", figma_access_token)
      .send()
      .unwrap()
      .text()
      .unwrap();

    let response: Value = serde_json::from_str(&response).unwrap();
    let document = response.as_object().unwrap().get("document").unwrap();

    let mut image_node_ids = Vec::new();
    let mut stack = vec![document.clone()];

    while let Some(node) = stack.pop() {
      if let Some(obj) = node.as_object() {
        if let Some(node_type) = obj.get("type").and_then(|t| t.as_str()) {
          if node_type == "IMAGE" {
            if let Some(id) = obj.get("id").and_then(|i| i.as_str()) {
              image_node_ids.push(id.to_string());
            }
          }
        }

        if let Some(fills) = obj.get("fills").and_then(|f| f.as_array()) {
          if fills.iter().any(|fill| {
            fill
              .as_object()
              .and_then(|f| f.get("type"))
              .and_then(|t| t.as_str())
              == Some("IMAGE")
          }) {
            if let Some(id) = obj.get("id").and_then(|i| i.as_str()) {
              image_node_ids.push(id.to_string());
            }
          }
        }

        if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
          stack.extend(children.iter().cloned());
        }
      }
    }

    image_node_ids
  }
}
