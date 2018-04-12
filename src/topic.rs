extern crate xml;

use std;
use std::io::prelude::*;
use std::process::exit;
use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use clap::ArgMatches;

use self::xml::reader::{EventReader, ParserConfig, XmlEvent};

#[derive(Debug)]
struct Topic {
    num: String,
    title: String,
    desc: String,
    narr: String,
}

impl Topic {
    pub fn to_trec_string(&self) -> String {
        format!(
            "<query>
<number>{}</number>
<text>
{}
</text>
</query>\n",
            self.num, self.title
        )
    }
}

pub fn main(matches: ArgMatches) {
    let out_dir = matches.value_of("output").unwrap();
    std::fs::create_dir_all(out_dir).unwrap_or_else(|e| {
        println!("{}", e.description());
        exit(1)
    });

    let index = matches.value_of("index").unwrap();

    let paths = matches.values_of("INPUT").unwrap().collect::<Vec<_>>();

    for path in &paths {
        if !Path::new(path).is_file() {
            println!("{} is not a file. Ignored.", path);
            continue;
        }

        print!("Processing {} ... ", path);
        std::io::stdout().flush().ok();
        convert(path, index, &out_dir).unwrap_or_else(|e| println!("{:?}", e));
        println!("done.");
    }
}

fn convert<P: AsRef<Path>>(path: P, index: &str, out_dir: P) -> Result<(), Box<Error>> {
    let out_name = path.as_ref().file_name().unwrap();
    let mut out_name = Path::new(out_name).to_path_buf();
    out_name.set_extension("trec.txt");
    let out_path = out_dir.as_ref().join(out_name);
    let out_file = File::create(out_path)?;
    let mut out_file = BufWriter::new(out_file);

    let file = File::open(&path)?;
    let file = BufReader::new(file);
    // let mut parser = EventReader::new(file);
    let mut parser = ParserConfig::new()
        .trim_whitespace(true)
        .create_reader(file);

    out_file.write_all("<parameters>\n".as_bytes()).ok();
    out_file
        .write_all(format!("<index>{}</index>\n", index).as_bytes())
        .ok();
    out_file
        .write_all(format!("<trecFormat>true</trecFormat>\n").as_bytes())
        .ok();
    loop {
        let mut e = parser.next();
        match e {
            Ok(XmlEvent::StartDocument { .. }) => {
                continue;
            }
            Ok(XmlEvent::EndDocument) => {
                break;
            }
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "top" {
                    let topic = parse_topic(&mut parser);
                    let topic = match topic {
                        Ok(topic) => topic,
                        Err(e) => {
                            println!("Error: {}", e);
                            continue;
                        }
                    };
                    out_file.write_all(topic.to_trec_string().as_bytes()).ok();
                    e = parser.next(); // end tag
                    match e {
                        Ok(XmlEvent::EndElement { name, .. }) => {
                            debug_assert!(name.local_name == "top")
                        }
                        Ok(_) => return Err("Unexpected tag".into()),
                        Err(e) => return Err(e.into()),
                    };
                } else {
                    println!("else");
                }
            }
            Ok(XmlEvent::Whitespace(..)) => {
                continue;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            Ok(e) => {
                println!("Unexpected stuff {:?}", e);
            }
        }
    }

    out_file.write_all("</parameters>".as_bytes()).ok();

    Ok(())
}

fn parse_topic<R: Read>(parser: &mut EventReader<R>) -> Result<Topic, Box<Error>> {
    let num = match parse_tag(parser) {
        Ok((tag, text)) => {
            debug_assert!(tag == "num");
            text.trim_left_matches("Number:").trim_left().to_string()
        }
        Err(e) => return Err(e),
    };

    let title = match parse_tag(parser) {
        Ok((tag, text)) => {
            debug_assert!(tag == "title");
            text
        }
        Err(e) => return Err(e),
    };

    let desc = match parse_tag(parser) {
        Ok((tag, text)) => {
            debug_assert!(tag == "desc");
            text.trim_left_matches("Description:")
                .trim_left()
                .to_string()
        }
        Err(e) => return Err(e),
    };

    let narr = match parse_tag(parser) {
        Ok((tag, text)) => {
            debug_assert!(tag == "narr");
            text.trim_left_matches("Narrative:").trim_left().to_string()
        }
        Err(e) => return Err(e),
    };

    Ok(Topic {
        num,
        title,
        desc,
        narr,
    })
}

fn parse_tag<R: Read>(parser: &mut EventReader<R>) -> Result<(String, String), Box<Error>> {
    let mut e = parser.next();
    let tag = match e {
        Ok(XmlEvent::StartElement { name, .. }) => name.to_string(),
        Ok(e) => return Err(format!("Unexpected tag: {:?}", e).into()),
        Err(e) => return Err(e.into()),
    };

    e = parser.next();
    let text = if let Ok(XmlEvent::Characters(data)) = e {
        data
    } else {
        String::new()
    };

    e = parser.next();
    match e {
        Ok(XmlEvent::EndElement { name, .. }) => debug_assert!(tag == name.to_string()),
        Ok(e) => return Err(format!("Unexpected tag: {:?}", e).into()),
        Err(e) => return Err(e.into()),
    };

    Ok((tag, text))
}
