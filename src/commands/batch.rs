use std::{ffi::OsStr, path::PathBuf};

use crate::scalper::{self, ParsedValue};
use crate::structure::{Config, ConfigFormat};
use anyhow::{anyhow, Ok, Result};
use clap::{value_parser, Parser};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;

/// Parse multiple data fields from a N resources defined in a config file
#[derive(Parser)]
pub struct Args {
    /// Path to the config file.
    #[arg(short, long, value_name = "PATH", value_parser = value_parser!(PathBuf))]
    path: PathBuf,

    /// (Optional) Parameters to be passed to the resources separated by spaces.
    ///
    /// Example:
    ///
    /// ```
    /// --params param1 param2 param3
    /// ```
    ///
    /// More complex example, if parameter is needed only for the first and the third resource:
    ///
    /// ```
    /// --params param1 _ param3
    /// ```
    ///
    /// In this case, you can pass any value for the second parameter, because it will be ignored.
    #[arg(long, num_args(0..))]
    params: Option<Vec<String>>,
    // TODO: Add --json option to output the data in JSON format
    // be listed in resource structure. If one parameter is missing, than print the error
    // message with the list of missing parameters.
}

pub async fn command(args: Args) -> Result<()> {
    if !args.path.exists() {
        return Err(anyhow!("File does not exist!"));
    }

    let config_format = match args.path.extension().and_then(OsStr::to_str) {
        Some("json") => ConfigFormat::Json,
        Some("toml") => ConfigFormat::Toml,
        _ => return Err(anyhow!("Invalid file format!")),
    };

    let mut config = Config::from_file(&args.path, config_format)?;

    if config.needs_parameters() {
        if args.params.is_none() {
            return Err(anyhow!(
                "This config needs parameters!\nMore info: rvp batch --help"
            ));
        }

        let params = args.params.unwrap();

        if config.resources.len() != params.len() {
            return Err(anyhow!(
                "The number of parameters does not match the number of resources!"
            ));
        }

        for (i, param) in params.iter().enumerate() {
            config.resources[i].mut_url_with_param(param);
        }
    }

    // TODO: parse in a thread pool
    for resource in config.resources {
        let parsed_values = scalper::grab(resource.selectors, &resource.url).await?;
        print_table(&parsed_values, &resource.url);
    }

    Ok(())
}

/// Print the parsed values in a table
fn print_table(parsed_values: &Vec<ParsedValue>, resource_url: &String) {
    println!("Table for resource:\n{}", resource_url);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Name", "Value"]);
    for parsed_value in parsed_values {
        table.add_row(vec![&parsed_value.name, &parsed_value.value]);
    }

    println!("{}", table);
}
