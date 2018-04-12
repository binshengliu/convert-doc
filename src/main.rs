#[macro_use]
extern crate clap;
extern crate serde_json;

mod washingtonpost;
mod topic;

use clap::{App, Arg};

fn main() {
    let matches = App::new("convert_doc")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A tool for converting collection format.")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .default_value(".")
                .value_name("DIRECTORY")
                .takes_value(true)
                .display_order(1)
                .help("Specify the output directory"),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .required(true)
                .value_name("TYPE")
                .takes_value(true)
                .possible_values(&["topic", "wp"])
                .display_order(2)
                .help("Specify the type of files to be converted"),
        )
        .arg(
            Arg::with_name("index")
                .short("i")
                .long("index")
                // .default_value(".")
                .value_name("DIRECTORY")
                .takes_value(true)
                .required_if("type", "topic")
                // .display_order(1)
                .help("Specify the directory for indexes"),
        )
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .multiple(true)
                .display_order(1000)
                .help("Specify files to convert"),
        )
        .get_matches();

    match matches.value_of("type") {
        Some("topic") => topic::main(matches),
        Some("wp") => washingtonpost::main(matches),
        _ => (),
    };
}
