use clap::{command, Arg};
use regex::Regex;
use std::process::Command;

fn main() -> Result<(), String> {
    let match_result = command!()
        .about("This program extracts a VDATA file from a VPK file and optionally converts it to JSON.")
        .arg(
            Arg::new("path_to_vpk")
                .required(true)
                .help("Path to VPK to be searched through."),
        )
        .arg(
            Arg::new("path_to_vdata")
                .required(true)
                .help("Path to VDATA inside VPK."),
        )
        .get_matches();

    let path_to_vpk = match_result
        .get_one::<String>("path_to_vpk")
        .unwrap()
        .to_string();
    let path_to_vdata = match_result
        .get_one::<String>("path_to_vdata")
        .unwrap()
        .to_string();

    let vdata = grab_vdata(path_to_vpk, path_to_vdata)?;
    let json = parse_vdata_to_json(vdata)?;

    println!("{}", json);
    Ok(())
}

fn grab_vdata(path_to_vpk: String, path_to_vdata: String) -> Result<String, String> {
    let output = Command::new("./Decompiler")
        .arg("-i")
        .arg(path_to_vpk)
        .arg("-f")
        .arg(path_to_vdata)
        .arg("-b")
        .arg("DATA")
        .output()
        .map_err(|e| format!("Could not find Decompiler: {}", e))?;

    // Unwrap here because whomever wrote the decompiler doesnt like errors and insteads likes bad
    // outputs
    Ok(String::from_utf8(output.stdout).unwrap())
}

fn parse_vdata_to_json(vdata: String) -> Result<String, String> {
    let assignment =
        Regex::new(r#"([a-zA-Z_0-9]+) = (?:("[^"]*"|true|false|[+-]?[\d]*\.?\d+|))"#).unwrap();

    let assignment_with_quotes =
        Regex::new(r#"("[^"]*") = (?:("[^"]*"|true|false|[+-]?[\d]*\.?\d+|))"#).unwrap();

    let braces = Regex::new(r#"(\[|\]|\{|\})"#).unwrap();
    let number_or_string = Regex::new(r#"([+-]?[\d]*\.?\d+|"[a-zA-Z0-9_+\.\| ]*")"#).unwrap();
    let bad_line_detector = Regex::new("soundevent:|panorama:|resource_name:").unwrap();

    let json = vdata
        .split_once(r#"--- Data for block "DATA" ---"#)
        .ok_or("VDATA file was formatted incorrectly. Probably because path to VPK or path to VDATA is incorrect.")?
        .1
        .chars()
        .collect::<String>()
        .lines()
        .skip(2)
        .map(|line| {
            // Removes all lines with `soundevent:`, `panorama:`, or `resource_name`.
            if bad_line_detector.captures(line).is_some() {
                return "".to_string();
            }

            if let Some((_, [var, assign])) = assignment.captures(line).map(|c| c.extract()) {
                return format!("\"{}\":{},", var, assign);
            };

            if let Some((_, [var, assign])) = assignment_with_quotes.captures(line).map(|c| c.extract()) {
                return format!("{}:{},", var, assign);
            }

            if let Some((_, [brace])) = braces.captures(line).map(|c| c.extract()) {
                return match brace {
                    "{" | "[" => brace.to_string(),
                    "}" => "},".to_string(),
                    "]" => "],".to_string(),
                    // Should never reach here
                    _ => "".to_string()
                };
            }

            if let Some((_, [item])) = number_or_string.captures(line).map(|c| c.extract()) {
                return format!("{},", item);
            }

            "".to_string()
        })
    .collect::<String>();

    // Correct trailing commas.
    let json = str::replace(&json, ",}", "}");
    let json = str::replace(&json, ",]", "]");
    // Clear edge case where variable is assigned to object
    let json = str::replace(&json, ",[", "[");
    let json = str::replace(&json, ",{", "{");

    // Fixes edge case where `},{` gets changed to `}{` in arrays.
    // Also for `],[` to `][`
    let json = str::replace(&json, "}{", "},{");
    let json = str::replace(&json, "][", "],[");

    // Get rid of last trailing commas
    let mut json = json.chars();
    json.next_back();
    let json = json.collect::<String>();

    // UNCOMMENT WHEN TESTING
    // This line 'fixes' all the shit i couldn't figure out. (Deletes Content)
    let json = str::replace(&json, ":,", ":\"\",");

    Ok(json)
}
