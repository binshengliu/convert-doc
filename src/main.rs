#[macro_use]
extern crate clap;
extern crate serde_json;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use serde_json::Value;

use clap::{App, Arg};

#[derive(Debug)]
struct Article {
    id: String,
    url: String,
    title: String,
    author: String,
    date: u64,
    fullcaption: String,
    text: String,
}

impl Article {
    pub fn to_trec_string(&self) -> String {
        format!(
            "<DOC>
<DOCNO>
{}
</DOCNO>
<URL>
{}
</URL>
<TITLE>
{}
</TITLE>
<AUTHOR>
{}
</AUTHOR>
<FULLCAPTION>
{}
</FULLCAPTION>
<TEXT>
{}
<TEXT>
</DOC>\n",
            self.id, self.url, self.title, self.author, self.fullcaption, self.text
        )
    }
}

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
            Arg::with_name("INPUT")
                .required(true)
                .multiple(true)
                .display_order(1000)
                .help("Specify files to convert"),
        )
        .get_matches();

    let out_dir = matches.value_of("output").unwrap();
    std::fs::create_dir_all(out_dir).unwrap_or_else(|e| {
        println!("{}", e);
        exit(1)
    });

    let paths = matches.values_of("INPUT").unwrap().collect::<Vec<_>>();

    for path in &paths {
        if !Path::new(path).is_file() {
            println!("{} is not a file. Ignored.", path);
            continue;
        }

        print!("Processing {} ... ", path);
        std::io::stdout().flush().ok();
        convert_json_file(path, &out_dir).unwrap_or_else(|e| println!("{:?}", e));
        println!("done.");
    }
}

fn convert_json_file<P: AsRef<Path>>(path: P, out_dir: P) -> Result<(), Box<Error>> {
    let out_name = path.as_ref().file_name().unwrap();
    let mut out_name = Path::new(out_name).to_path_buf();
    out_name.set_extension("trec.txt");
    let out_path = out_dir.as_ref().join(out_name);
    let out_file = File::create(out_path)?;
    let mut out_file = BufWriter::new(out_file);

    let file = File::open(&path)?;
    let line_reader = BufReader::new(file);
    for line in line_reader.lines() {
        let line = line?;
        let article = parse_article(serde_json::from_str(&line)?);
        let output = article.to_trec_string();
        out_file.write_all(output.as_bytes()).ok();
    }
    Ok(())
}

fn parse_article(json: Value) -> Article {
    // let contents = json["contents"].as_array().unwrap();
    let id = json["id"].as_str().unwrap_or("");
    let url = json["article_url"].as_str().unwrap_or("");
    let title = json["title"].as_str().unwrap_or("");
    let author = json["author"].as_str().unwrap_or("");
    let date = json["published_date"].as_u64().unwrap_or(0);

    // contents part
    let mut text = Vec::new();
    let mut fullcaption = Vec::new();
    if let Value::Array(ref contents) = json["contents"] {
        for content in contents.iter() {
            if let &Value::Object(ref entry) = content {
                if entry.contains_key("fullcaption") {
                    fullcaption.push(entry["fullcaption"].as_str().unwrap_or(""));
                }

                if entry.contains_key("content") {
                    let content_str = entry["content"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or(entry["content"].as_u64().map(|u| u.to_string()))
                        .unwrap_or("".to_string());
                    text.push(content_str);
                }
            }
        }
    }
    let text = text.join(" ");
    let fullcaption = fullcaption.join(" ");

    // println!("id: {}", id);
    // println!("url: {}", url);
    // println!("title: {}", title);
    // println!("author: {}", author);
    // println!("date: {}", date);
    // println!("fullcaption: {}", fullcaption);
    // println!("text: {}", text);
    Article {
        id: id.to_string(),
        url: url.to_string(),
        title: title.to_string(),
        author: author.to_string(),
        date: date,
        fullcaption: fullcaption,
        text: text,
    }
}
