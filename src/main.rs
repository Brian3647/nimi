use std::collections::HashMap;

use clap::Parser;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Context;
use colored::ColoredString;
use colored::Colorize;
use isahc::ReadResponseExt;
use linku_sona::{UsageCategory, Word};

mod cache;
mod config;

use color_eyre::Result;
use config::Config;
use serde_json::Value;

#[derive(serde::Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
enum ApiResult {
	Word(Word),
	Error { message: String },
}

impl From<ApiResult> for Result<Word> {
	fn from(val: ApiResult) -> Self {
		match val {
			ApiResult::Word(word) => Ok(word),
			ApiResult::Error { message } => Err(eyre!("Api error: {}", message)),
		}
	}
}

#[derive(Parser)]
struct Cli {
	/// Show the RAW JSON response from the API
	#[clap(short = 'j', long)]
	json: bool,
	/// The language used to get the word definitions.
	#[clap(short = 't', long)]
	toki: Option<String>,
	/// The word to get the definition of
	word: String,
}

fn main() -> Result<()> {
	color_eyre::install()?;
	let cli = Cli::parse();
	let cfg = Config::get_config().wrap_err("Failed to get config")?;
	let toki = match cli.toki {
		None => cfg.toki,
		Some(toki) => toki,
	};

	let json = match cache::get_from_cache(&toki, cfg.cache_lifetime_seconds)? {
		None => {
			let url = format!("https://api.linku.la/v1/words?lang={}", toki);
			let response = isahc::get(&url)
				.wrap_err("Failed to download data")?
				.text()
				.wrap_err("Downloaded data is not text")?;
			let json: Value = serde_json::from_str(&response)
				.wrap_err_with(|| format!("Failed to parse response string: {}", response))?;
			cache::write_to_cache(&toki, &json)?;
			json
		}
		Some(result) => result,
	};

	let json = &json[cli.word];
	if cli.json {
		println!("{json}");
		return Ok(());
	}

	let word: Word = serde_json::from_value(json.clone())
		.wrap_err_with(|| format!("Failed to get api result: {}", json))?;
	show(word, toki);
	Ok(())
}

fn show(word: Word, toki: String) {
	let translations = word.translations;
	let definition = translations
		.get(&toki)
		.map(|t| t.definition.clone())
		.unwrap_or_else(|| {
			format!(
				"No definition found for \"{}\" with language code \"{}\".",
				word.word, toki
			)
		});

	println!(
		"{} {} {}",
		"~>".bold(),
		word.word.bold(),
		if let Some(ucsur) = word.representations.and_then(|r| r.ucsur) {
			char::from_u32(u32::from_str_radix(ucsur.trim_start_matches("U+"), 16).unwrap())
				.unwrap()
		} else {
			' '
		}
	);

	println!(
		"{} {} {} - {}",
		colored_usage_category(&word.usage_category),
		format!("({}%) ·", get_usage_percentage(word.usage)).bright_black(),
		word.book.to_string().bright_black(),
		word.creator.join(", ")
	);

	println!("--------------");
	println!("{}", definition)
}

fn colored_usage_category(cat: &UsageCategory) -> ColoredString {
	match cat {
		UsageCategory::Core => "core".green(),
		UsageCategory::Common => "common".yellow(),
		UsageCategory::Uncommon => "uncommon".red(),
		UsageCategory::Obscure => "obscure".magenta(),
		UsageCategory::Sandbox => "sandbox".bright_black(),
	}
}

fn get_usage_percentage(usage: HashMap<String, u8>) -> u8 {
	let mut keys = usage.keys().collect::<Vec<_>>();
	keys.sort();
	keys.last()
		.and_then(|k| usage.get(*k).copied())
		.unwrap_or(0)
}
