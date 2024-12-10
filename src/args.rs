use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
	#[clap(short, long)]
	pub input: PathBuf,
	pub files: Vec<String>,
}

pub fn parse_path_arg(arg: &str) -> Result<(PathBuf, Option<u8>)> {
	let arg = arg.trim();
	match arg.split_once('=') {
		Some((path, z)) => {
			let path = PathBuf::from(path.trim());
			let z = z
				.trim()
				.parse::<u8>()
				.map(Some)
				.wrap_err_with(|| format!("failed to parse '{z}' as numeric z-level"))?;
			Ok((path, z))
		}
		None => Ok((PathBuf::from(arg), None)),
	}
}
