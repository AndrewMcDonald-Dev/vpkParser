use clap::{command, Arg, ArgGroup, ArgMatches};

mod txt_parser;
mod vdata_parser;
use crate::txt_parser::process_txt_parser;
use crate::vdata_parser::process_vdata_parser;

fn generate_args() -> ArgMatches {
    command!()
        .subcommand(
            clap::Command::new("vdata_parser")
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
                    .arg(
                    Arg::new("skip_json")
                        .short('s')
                        .action(clap::ArgAction::SetTrue)
                        .help("When set, parsing of VDATA to JSON will be skipped and VDATA will be outputted.")
                )
        )
        .subcommand(
            clap::Command::new("local_parser")
                .about("This program extracts a localization text file to JSON.")
                .arg(
                    Arg::new("path_to_txt")
                        .short('f')
                        .long("file")
                        .help("Path to text file to be parsed to JSON.")
                        .conflicts_with("path_to_txt_folder")
                )
                    .arg(
                    Arg::new("path_to_txt_folder")
                        .short('d')
                        .long("folder")
                        .help("Path to folder of text files to be parsed to JSON.")
                        .conflicts_with("path_to_txt")
                )
                    .arg(
                    Arg::new("skip_json")
                        .short('s')
                        .action(clap::ArgAction::SetTrue)
                        .help("When set, parsing of txt to JSON will be skipped and txt file will be outputted.")
                        .conflicts_with("path_to_txt_folder")
                )
                    .arg(
                    Arg::new("keep_empty_files")
                        .short('k')
                        .action(clap::ArgAction::SetTrue)
                        .help("When set, localization files which are empty will return empty objects instead of not appearing.")
                )
                    .arg(
                    Arg::new("sub_folders")
                        .short('g')
                        .long("sub_folders")
                        .help("Optional setting reading path_to_txt_folder as the parent localization folder and then using this command to describe several specific localization folders.")
                        .num_args(1..=7)
                        .requires("path_to_txt_folder")
                    )
                .group(
                    ArgGroup::new("path")
                        .args(["path_to_txt", "path_to_txt_folder"])
                        .required(true)

                )
        )
        .get_matches()
}

fn main() -> Result<(), String> {
    let match_result = generate_args();
    let output = match match_result.subcommand() {
        Some((command, matches)) => match command {
            "vdata_parser" => process_vdata_parser(matches)?,
            "local_parser" => process_txt_parser(matches)?,
            _ => panic!("Bad command should never reach here."),
        },
        None => panic!("Bad arguments should never reach here."),
    };

    println!("{}", output);

    Ok(())
}
