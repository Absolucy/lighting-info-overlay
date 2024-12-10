use color_eyre::eyre::{Result, WrapErr};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, path::Path};

pub type LumCount = Option<f32>;
pub type ZLighting = Vec<Vec<LumCount>>;

#[serde_as]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum LightingInfo {
	Single(ZLighting),
	Multi(#[serde_as(as = "HashMap<DisplayFromStr, _>")] HashMap<u8, ZLighting>),
}

impl LightingInfo {
	pub fn get_info_for_z(&self, z: Option<u8>) -> Option<&ZLighting> {
		match (self, z) {
			(Self::Single(info), _) => Some(info),
			(Self::Multi(info), _) if info.len() == 1 => Some(info.values().next().unwrap()),
			(Self::Multi(info), Some(z)) => info.get(&z),
			(Self::Multi(_), None) => None,
		}
	}
}

pub fn read_info(path: &Path) -> Result<LightingInfo> {
	let file_contents = std::fs::read_to_string(path).wrap_err("failed to read file")?;
	serde_json::from_str(&file_contents).wrap_err("failed to parse JSON")
}
