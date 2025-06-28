use color_eyre::eyre::{Result, WrapErr};
use serde::Deserialize;
use std::{ops::Deref, path::Path};

pub type LumCount = Option<f32>;
pub type ZLighting = Vec<Vec<LumCount>>;

#[derive(Clone, PartialEq, Deserialize)]
pub struct LightingInfo(Vec<ZLighting>);

impl LightingInfo {
	pub fn get_info_for_z(&self, z: Option<u8>) -> Option<&ZLighting> {
		match z {
			Some(z) => self.0.get(z as usize + 1),
			None => self.0.first(),
		}
	}
}

impl AsRef<Vec<ZLighting>> for LightingInfo {
	fn as_ref(&self) -> &Vec<ZLighting> {
		&self.0
	}
}

impl Deref for LightingInfo {
	type Target = Vec<ZLighting>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub fn read_info(path: &Path) -> Result<LightingInfo> {
	let file_contents = std::fs::read_to_string(path).wrap_err("failed to read file")?;
	serde_json::from_str(&file_contents).wrap_err("failed to parse JSON")
}
