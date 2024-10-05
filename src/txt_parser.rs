use std::fs;

use clap::ArgMatches;
use regex::Regex;

pub fn process_txt_parser(matches: &ArgMatches) -> Result<String, String> {
    let path_to_vpk = matches
        .get_one::<String>("path_to_txt")
        .unwrap()
        .to_string();

    let mut output: String = fs::read_to_string(path_to_vpk).map_err(|e| {
        format!(
            "Path to text file was incorrect or could not be read. {}",
            e
        )
    })?;

    if !matches.get_flag("skip_json") {
        output = clean_txt_file(output)?;
        output = parse_txt_to_json(output)?;
    }

    Ok(output)
}

fn parse_txt_to_json(output: String) -> Result<String, String> {
    println!("{}", output);
    Ok("".to_string())
}

fn clean_txt_file(output: String) -> Result<String, String> {
    let cleaner =
        Regex::new(r"(?:\s*//-+\s*)+// (?P<hero>(?:[\S ][^-])+)(?:\s*// [\S ]+)?(?:\s*//-+\s*)+")
            .unwrap();

    Ok(cleaner.replace_all(&output, "\n${hero}\n").to_string())
}
