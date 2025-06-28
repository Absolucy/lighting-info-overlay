pub mod info;

use crate::info::ZLighting;
use clap::Parser;
use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
use image::{ImageFormat, Rgba, RgbaImage};
use std::path::PathBuf;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser)]
pub struct Args {
	#[clap(short, long)]
	pub input: PathBuf,
	pub files: Vec<String>,
}

fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let info = info::read_info(&args.input).wrap_err_with(|| {
		format!(
			"failed to read lighting info from {path}",
			path = args.input.display()
		)
	})?;

	let files = match (info.len(), args.files.len()) {
		(1, 1) => args.files,
		(x, 1) => (1..=x)
			.map(|z| args.files[0].replace("$z", &z.to_string()))
			.collect(),
		(x, y) if x == y => args.files,
		_ => {
			return Err(eyre!(
				"You must either give one file per z-level as input, or a single file where $z \
				 will be replaced with the z-level number."
			))
		}
	};

	for (info, path) in info.iter().zip(files.iter()) {
		handle_map(info, path).wrap_err_with(|| format!("failed to handle {path}"))?;
	}

	Ok(())
}

fn handle_map(info: &ZLighting, path: &str) -> Result<()> {
	const FULLDARK_ALPHA: f32 = 256.0 * 0.8;

	let path = PathBuf::from(path);
	let mut image = image::open(&path)
		.wrap_err("failed to open image")?
		.to_rgba8();
	let width = info.len() as u32;
	for (x, col) in info.iter().enumerate() {
		let height = col.len() as u32;
		for (y, lumcount) in col
			.iter()
			.enumerate()
			.filter_map(|(y, lumcount)| lumcount.map(|l| (y, l)))
		{
			assert!(lumcount >= 0.0, "lumcount {lumcount} below 0");
			assert!(lumcount <= 1.0, "lumcount {lumcount} above 1");
			if lumcount >= 0.9 {
				continue;
			}
			let alpha = FULLDARK_ALPHA - (FULLDARK_ALPHA * lumcount);
			if alpha <= f32::EPSILON {
				continue;
			}
			let alpha = alpha.round() as u8;
			draw_square(&mut image, x as u32, y as u32, width, height, alpha).wrap_err_with(
				|| format!("failed to draw to pos {x},{y} (alpha={alpha}, lumcount={lumcount})"),
			)?;
		}
	}
	let original_filename = path
		.file_stem()
		.wrap_err("path did not have filename")?
		.to_str()
		.wrap_err("path did not have valid UTF-8 filename")?;
	let save_path = path.with_file_name(format!("{original_filename}.lighting.png"));
	image
		.save_with_format(&save_path, ImageFormat::Png)
		.wrap_err_with(|| {
			format!(
				"failed to save output image to {path}",
				path = save_path.display()
			)
		})?;

	Ok(())
}

fn draw_square(
	image: &mut RgbaImage,
	grid_x: u32,
	grid_y: u32,
	grid_width: u32,
	grid_height: u32,
	square_alpha: u8,
) -> Result<(), image::ImageError> {
	let width = image.width();
	let height = image.height();
	let cell_width = width / grid_width;
	let cell_height = height / grid_height;

	let start_x = grid_x * cell_width;
	let start_y = height - ((grid_y + 1) * cell_height);

	let square_color = Rgba([0, 0, 0, square_alpha]);
	for x in start_x..(start_x + cell_width).min(width - 1) {
		for y in start_y..(start_y + cell_height).min(height - 1) {
			let existing_pixel = image.get_pixel(x, y);
			let blended_pixel = blend_pixels(existing_pixel, square_color);
			image.put_pixel(x, y, blended_pixel);
		}
	}

	Ok(())
}

fn blend_pixels(bottom: &Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
	let a_bottom = bottom[3] as f32 / 255.0;
	let a_top = top[3] as f32 / 255.0;

	// Compute output alpha
	let a_out = a_top + a_bottom * (1.0 - a_top);

	// Compute blended color channels
	let r_out = ((top[0] as f32 * a_top + bottom[0] as f32 * a_bottom * (1.0 - a_top)) / a_out)
		.clamp(0.0, 255.0) as u8;
	let g_out = ((top[1] as f32 * a_top + bottom[1] as f32 * a_bottom * (1.0 - a_top)) / a_out)
		.clamp(0.0, 255.0) as u8;
	let b_out = ((top[2] as f32 * a_top + bottom[2] as f32 * a_bottom * (1.0 - a_top)) / a_out)
		.clamp(0.0, 255.0) as u8;
	let a_out_u8 = (a_out * 255.0).clamp(0.0, 255.0) as u8;

	Rgba([r_out, g_out, b_out, a_out_u8])
}
