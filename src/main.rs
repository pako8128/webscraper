#[macro_use]
extern crate clap;
extern crate hyper;
extern crate futures;

use clap::App;

use hyper::Client;
use hyper::rt;
use hyper::client::HttpConnector;

use futures::{stream, Future, Stream};

use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::{File, DirBuilder};

fn main() {
	// parse the command line arguments
	let yaml = load_yaml!("../args.yml");
	let matches = App::from_yaml(yaml).get_matches();
	
	let urls = match matches.values_of("URLS") {
	    Some(urls) => urls.collect(),
	    None => vec!["http://example.com"],
	};

    let output_dir = match matches.value_of("output") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("."),
    };

    // create the directory if it doesn't exist already
    if !output_dir.exists() {
        DirBuilder::new().recursive(true).create(&output_dir).unwrap();
    }
    
	let urls = urls.into_iter().map(|url| url.parse().unwrap()).collect();
   
	// using the tokio runtime
	rt::run(fetch_urls(urls, output_dir));
}

fn fetch_urls(urls: Vec<hyper::Uri>, output_dir: PathBuf) -> impl Future<Item=(), Error = ()> {
    let client = Client::new();

    stream::iter_ok(urls)
        .for_each(move |url| {
            let mut output_dir = output_dir.join(url.path().split('/').last().unwrap());
            if output_dir.is_dir() {
                output_dir.push("index.html");
            }
            println!("output_dir: {:?}", output_dir);
            let output = File::create(output_dir).unwrap();
            fetch_url(url, &client, output)
        })
}

fn fetch_url(url: hyper::Uri, client: &Client<HttpConnector>, mut output: File) -> impl Future<Item = (), Error = ()> {
    client
        .get(url)
        .and_then(move |res| {
            println!("Status: {}", res.status());
            println!("Header: {:#?}", res.headers());

            res.into_body()
                .for_each(move |chunk| {
                    output
                    .write_all(&chunk)
                    .map_err(|_| panic!("oops"))
                })
        })
        .map_err(|err| {
            println!("Error: {:?}", err);
        })
}
