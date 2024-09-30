use std::process::Command;

fn main() -> Result<(), String> {
    let path_to_vpk = std::env::args().nth(1).ok_or("Bad path to VPK.\n Remember, correct formatting is vpkParser [path to vpk] [path to vdata]")?;
    let path_to_vdata = std::env::args().nth(2).ok_or("Bad path to VDATA.\n Remember, correct formatting is vpkParser [path to vpk] [path to vdata]")?;

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
    let output: String = vdata
        .split_once(r#"--- Data for block "DATA" ---"#)
        .ok_or("VDATA file was formatted incorrectly. Probably because path to VPK or path to VDATA is incorrect.")?
        .1
        .lines()
        .skip(2)
        .map(|line| line.chars().filter(|c| c != &'\t').collect::<String>())
        .map(|line| str::replace(&line.clone(), " = ", ":"))
        .collect();

    Ok(output)
}
