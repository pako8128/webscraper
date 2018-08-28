#[macro_use]
extern crate clap;
extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate tokio;

use chrono::prelude::*;
use clap::App;

use hyper::client::HttpConnector;
use hyper::rt;
use hyper::Client;

use futures::{stream, Future, Stream};

use tokio::prelude::*;

use std::fs::{DirBuilder, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::process;

fn main() {
    // parse the command line arguments
    let yaml = load_yaml!("../args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut urls = Vec::new();

    if let Some(url_file) = matches.value_of("url-list") {
        use std::io::BufRead;

        let file = BufReader::new(File::open(&url_file).unwrap());
        for url in file.lines() {
            let url = url.unwrap();
            urls.push(url.parse().unwrap());
        }
    }

    match matches.values_of("URLS") {
        Some(u) => {
            for url in u {
                urls.push(url.parse().unwrap());
            }
        }
        None => {
            if urls.is_empty() {
                println!("{}", matches.usage());
                process::exit(1);
            }
        }
    }

    let output_dir = match matches.value_of("output") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("."),
    };

    // create the directory if it doesn't exist already
    if !output_dir.exists() {
        DirBuilder::new()
            .recursive(true)
            .create(&output_dir)
            .unwrap();
    }

    // using the tokio runtime
    rt::run(fetch_urls(urls, output_dir));
}

fn fetch_urls(urls: Vec<hyper::Uri>, output_dir: PathBuf) -> impl Future<Item = (), Error = ()> {
    let client = Client::new();

    stream::iter_ok(urls).for_each(move |url| {
        let filename = output_dir.join(&construct_filename(&url));
        let output = File::create(&filename).unwrap();
        fetch_url(url, &client, output)
    })
}

fn fetch_url(
    url: hyper::Uri,
    client: &Client<HttpConnector>,
    mut output: File,
) -> impl Future<Item = (), Error = ()> {
    client
        .get(url)
        .and_then(move |res| {
            println!("Status: {}", res.status());
            println!("Header: {:#?}", res.headers());

            res.into_body()
                .for_each(move |chunk| output.write_all(&chunk).map_err(|_| panic!("oops")))
        })
        .map_err(|err| {
            println!("Error: {:?}", err);
        })
}

fn construct_filename(url: &hyper::Uri) -> String {
    let mut filename = format!("{}", url).split_at(7).1.replace('/', "_");
    if filename.ends_with('_') {
        filename.pop().unwrap();
    }
    let now = Local::now();
    format!(
        "{}.{}.{}.{}_{}:{}:{}",
        filename,
        now.day(),
        now.month(),
        now.year(),
        now.hour(),
        now.minute(),
        now.second()
    )
}
