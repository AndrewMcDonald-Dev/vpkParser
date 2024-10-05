use clap::{command, Arg, ArgMatches};

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
                        .required(true)
                        .help("Path to text file to be parsed to JSON.")
                )
                    .arg(
                    Arg::new("skip_json")
                        .short('s')
                        .action(clap::ArgAction::SetTrue)
                        .help("When set, parsing of txt to JSON will be skipped and txt file will be outputted.")
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

// (?:\s*//-+\s*){2,}// (?P<hero>[\S ][^-]+)\n\s*// (?:[\S ]+)?(?:\s*//-+\s*)+
// This regex will be used for the localization parser
