use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::{fs, time::SystemTime};

fn get_cachefile_path(toki: &str) -> Result<std::path::PathBuf> {
	let mut path = dirs::cache_dir().wrap_err("Failed to get cache path")?;
	path.push("seme");
	fs::create_dir_all(&path).wrap_err("Failed to create cache directory")?;
	path.push(format!("{toki}.json"));
	Ok(path)
}

pub fn get_from_cache(toki: &str, cache_lifetime_seconds: u64) -> Result<Option<Value>> {
	let path = get_cachefile_path(toki)?;
	let file = match File::open(path) {
		Ok(file) => Ok(file),
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
		Err(e) => Err(e),
	}?;

	let modified = file
		.metadata()
		.wrap_err("Failed to get cache file metadata")?
		.modified()
		.wrap_err("Failed to get file modification date")?;
	let cache_still_valid =
		SystemTime::now().duration_since(modified)?.as_secs() <= cache_lifetime_seconds;

	let result = cache_still_valid
		.then(|| get_json_from_file(file))
		.transpose()?;
	Ok(result)
}

fn get_json_from_file(file: File) -> Result<Value> {
	let reader = BufReader::new(file);
	serde_json::from_reader(reader).wrap_err("Failed to parse cache file as json")
}

pub fn write_to_cache(language: &str, value: &Value) -> Result<()> {
	fs::write(
		get_cachefile_path(language)?,
		serde_json::to_string(value)
			.wrap_err_with(|| format!("Invalid response to add to cache: {}", value))?,
	)
	.wrap_err("Failed to write cache")
}
