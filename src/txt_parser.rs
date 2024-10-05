use std::{
    fs::File,
    io::{BufReader, Read},
};

use clap::ArgMatches;
use regex::Regex;
use unicode_bom::Bom;

pub fn process_txt_parser(matches: &ArgMatches) -> Result<String, String> {
    let path_to_vpk = matches
        .get_one::<String>("path_to_txt")
        .unwrap()
        .to_string();

    let mut file = File::open(path_to_vpk).map_err(|e| format!("Failed to open file: {}", e))?;

    let bom = Bom::from(&mut file);
    let mut reader = BufReader::new(file);

    if bom != Bom::Null {
        let mut x = [0; 0];
        let _y = reader.read_exact(&mut x);
    }

    let mut output = "".to_string();
    reader
        .read_to_string(&mut output)
        .map_err(|e| format!("Failed to read rest of file: {}", e))?;

    if !matches.get_flag("skip_json") {
        output = clean_txt_file(output)?;
        output = parse_txt_to_json(output)?;
    }

    Ok(output)
}

fn parse_txt_to_json(input: String) -> Result<String, String> {
    let mut output = "{\"lang\":".to_string();

    let braces = Regex::new(r#"(\[|\]|\{|\})"#).unwrap();
    let hero = Regex::new(r#"(^[^"{}\[\]\t\n\r]+)"#).unwrap();
    let duo = Regex::new(r#"("[\S]+")[\s&&[^\n]]*("[\S ]+")"#).unwrap();
    let solo = Regex::new(r#"("[\S]+")"#).unwrap();

    let mut count = 0;

    input.lines().for_each(|line| {
        // Comments get thrown out
        if line.trim().starts_with('/') {
            return;
        }
        count += 1;

        if let Some((_, [left, right])) = duo.captures(line).map(|c| c.extract()) {
            // Process duo
            let item = left.to_string() + ":" + &escape_string(right) + ",";
            output.push_str(&item);
        } else if hero.captures(line).is_some() {
            if count != 1 {
                // Process hero
                output.push_str(&("},\"".to_string() + line + "\":{"));
            }
        } else if let Some((_, [item])) = solo.captures(line).map(|c| c.extract()) {
            // Process solo
            output.push_str(&(item.to_string() + ":"));
        } else if let Some((_, [brace])) = braces.captures(line).map(|c| c.extract()) {
            // Process braces
            let brace = match brace {
                "{" | "[" => brace,
                "}" => "},",
                "]" => "],",
                // Should never reach here
                _ => "",
            };
            output.push_str(brace);
        }
    });

    // Remove first instance of `},` cause of bad code.
    let output = output.replacen("},", "", 1);
    // Correct trailing commas.
    let output = str::replace(&output, ",}", "}");
    let output = &output[0..output.len() - 1];
    let output = str::replace(output, r"\\\", r"\");

    // Current problem:
    // When pushing a hero string inside another hero string we gain an extra `},`. This is only
    // done once in the hero text file and the correction is hardcoded.
    // Also, every layer down we go we need another closing brace at the end of the outer object.
    // For heros text that is at the very end of the file and it needs only 1. The 2nd `}` is for
    // the initial `{` i added at the beginning.

    Ok(output + "}}")
}

fn clean_txt_file(input: String) -> Result<String, String> {
    let cleaner =
        Regex::new(r"(?:\s*//-+\s*)+// (?P<hero>(?:[\S ][^-])+)(?:\s*// [\S ]+)?(?:\s*//-+\s*)+")
            .unwrap();

    Ok(cleaner.replace_all(&input, "\n${hero}\n").to_string())
}

fn escape_string(value: &str) -> String {
    let value = str::replace(value, r"\", r"\\");
    let value = &value[1..value.len() - 1];
    let value = str::replace(value, r#"""#, r#"\""#);
    "\"".to_string() + &str::replace(&value, r"/", r"\/") + "\""
}
