extern crate serde_json;

use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::io::prelude::*;
use std::path::Path;

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
    let paths = vec![
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2012.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2013.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2014.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2015.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2016.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_article_2017.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2012.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2013.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2014.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2015.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2016.txt",
        "/research/remote/collections/TREC/WashingtonPost/data/TREC_blog_2017.txt",
    ];
    for path in &paths {
        print!("Processing {} ... ", path);
        std::io::stdout().flush().ok();
        convert_json_file(path).unwrap_or_else(|e| println!("{:?}", e));
        println!("done.");
    }
}

fn convert_json_file<P: AsRef<Path>>(path: P) -> Result<(), Box<Error>> {
    let outpath = path.as_ref().to_path_buf();
    let outpath = outpath.file_name().unwrap();
    let mut outpath = Path::new(outpath).to_path_buf();
    outpath.set_extension("trec");
    let outfile = File::create(outpath)?;
    let mut outfile = BufWriter::new(outfile);

    let file = File::open(path)?;
    let line_reader = BufReader::new(file);
    for line in line_reader.lines() {
        let line = line?;
        let article = parse_article(serde_json::from_str(&line)?);
        let output = article.to_trec_string();
        outfile.write_all(output.as_bytes()).ok();
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
