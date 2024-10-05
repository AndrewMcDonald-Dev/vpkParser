use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use clap::ArgMatches;
use regex::Regex;
use unicode_bom::Bom;

pub fn process_txt_parser(matches: &ArgMatches) -> Result<String, String> {
    if matches.contains_id("path_to_txt") {
        process_single_txt_file(matches)
    } else {
        process_multiple_txt_files(matches)
    }
}

fn process_multiple_txt_files(matches: &ArgMatches) -> Result<String, String> {
    let folder = matches
        .get_one::<String>("path_to_txt_folder")
        .unwrap()
        .to_string();

    let keep_empty = matches.get_flag("keep_empty_files");

    if matches.contains_id("sub_folders") {
        let folder = Path::new(&folder);
        if folder.file_name().unwrap().ne("localization") {
            return Err("Misuse of sub_folders. The path_to_txt_folder should point to the localization folder if sub_folders is set.".to_string());
        }

        let sub_folders = matches
            .get_many::<String>("sub_folders")
            .unwrap()
            .map(|v| v.as_str())
            .collect::<Vec<_>>();

        let mut output = r#"{"folders":["#.to_string()
            + &sub_folders.iter().fold("".to_string(), |acc, item| {
                if acc.is_empty() {
                    format!("{}\"{}\"", acc, item)
                } else {
                    format!("{},\"{}\"", acc, item)
                }
            })
            + "],";

        for sub_folder in sub_folders {
            let path = folder.join(sub_folder);

            output.push_str(&("\"".to_string() + sub_folder + "\":"));
            let folder_contents = process_folder(path.to_str().unwrap().to_string(), keep_empty)?;
            output.push_str(&(folder_contents + ","));
        }

        let mut output = output[0..output.len() - 1].to_string();
        output.push('}');

        Ok(output.to_string())
    } else {
        process_folder(folder, keep_empty)
    }
}

fn process_folder(folder: String, keep_empty: bool) -> Result<String, String> {
    let paths = fs::read_dir(folder.clone())
        .map_err(|e| format!("Could not access files in folder: {}", e))?;

    let mut output = "{".to_string()
        + "\"category\":\""
        + Path::new(&folder).file_name().unwrap().to_str().unwrap()
        + "\","
        + "\"languages\":[";

    for path in paths {
        let path = path.map_err(|e| format!("Empty path given: {}", e))?.path();
        if path
            .extension()
            .ok_or("Non text file found in given folder.")?
            .ne("txt")
        {
            return Err("Non text file found in given folder.".to_string());
        }

        let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let bom = Bom::from(&mut file);
        let mut reader = BufReader::new(file);
        if bom != Bom::Null {
            let mut x = [0; 0];
            let _y = reader.read_exact(&mut x);
        }

        let mut json = "".to_string();
        reader
            .read_to_string(&mut json)
            .map_err(|e| format!("Failed to read rest of file: {}", e))?;

        json = clean_txt_file(json)?;
        let (json, should_add_comma) = parse_txt_to_json(json, keep_empty)?;

        output.push_str(&json);
        if should_add_comma {
            output.push(',');
        }
    }

    let mut output = output[0..output.len() - 1].to_string();

    output.push_str("]}");

    Ok(output)
}

fn process_single_txt_file(matches: &ArgMatches) -> Result<String, String> {
    let path_to_vpk = matches
        .get_one::<String>("path_to_txt")
        .unwrap()
        .to_string();

    // All this code to get rid of the BOM <feff>
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
        (output, _) = parse_txt_to_json(output, true)?;
    }

    Ok(output)
}

// Return type is (String, bool). The bool is to tell the calling function whether or not an empty
// file was passed. If true a comma is added (Note: if the file is empty but keep_empty is true then
// the comma should still be added).
fn parse_txt_to_json(input: String, keep_empty: bool) -> Result<(String, bool), String> {
    let mut output = "{\"lang\":".to_string();

    let braces = Regex::new(r#"(\[|\]|\{|\})"#).unwrap();
    let hero = Regex::new(r#"(^[^"{}\[\]\t\n\r]+)"#).unwrap();
    // The second space in the last capture group is intentional it is a different character
    let duo = Regex::new(r#"("[\S]+")[\s&&[^\n]]*("[\S  }]*")"#).unwrap();
    let solo = Regex::new(r#"("[\S]+")"#).unwrap();

    let mut count = 0;
    let mut hero_count = 0;

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
                hero_count += 1;
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
    let output = output.replace(r#""Tokens":{},"#, r#""Tokens":{"#);
    // Correct trailing commas.
    let output = output.replace(",}", "}");
    let output = &output[0..output.len() - 1];
    let output = output.replace(r"\\\", r"\");

    // Actually the most scuffed thing i do in this project
    // Some citadel_mods.txt files dont have the starting topic `Uprades: Weapon` so imma add it.
    let output = output.replace(
        r#""Tokens":{"AmmoPerSoul"#,
        r#""Tokens":{"Upgrades: Weapon":{"AmmoPerSoul"#,
    );
    let output = output.replace(
        r#""Tokens":{"Spirit"#,
        r#""Tokens":{"Upgrades: Weapon":{"Spirit"#,
    );
    let output = output.replace(
        r#""Tokens":{"MODIFIER"#,
        r#""Tokens":{"Unit states (that we want to show)":{"MODIFIER"#,
    );

    if hero_count == 0 && !keep_empty {
        Ok(("".to_string(), false))
    } else {
        Ok((output + "}}", true))
    }
}

fn clean_txt_file(input: String) -> Result<String, String> {
    let cleaner =
        Regex::new(r"(?:\s*//-+\s*)+// (?P<hero>(?:[\S ][^-])+)(?:\s*// [\S ]+)?(?:\s*//-+\s*)+")
            .unwrap();

    // Fixes a mistake on valves end
    let mistake_fix = Regex::new(r#"(?<left>"[\S]+")[\s&&[^\n]]*"\r?\n(?<right>[\S ]*")"#).unwrap();
    let input = mistake_fix.replace_all(&input, "${left}\"${right}");

    Ok(cleaner.replace_all(&input, "\n${hero}\n").to_string())
}

fn escape_string(value: &str) -> String {
    let value = str::replace(value, r"\", r"\\");
    let value = &value[1..value.len() - 1];
    let value = str::replace(value, r#"""#, r#"\""#);
    "\"".to_string() + &str::replace(&value, r"/", r"\/") + "\""
}
