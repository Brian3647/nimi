use color_eyre::Result;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
	pub toki: String,
	pub cache_lifetime_seconds: u64,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			toki: String::from("en"),
			cache_lifetime_seconds: 2_628_288, // 1 month
		}
	}
}

impl Config {
	pub fn get_config() -> Result<Self> {
		Ok(confy::load("seme", "config")?)
	}
}
