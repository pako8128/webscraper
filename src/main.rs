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
use std::io;

fn main() {
	// parse the command line arguments
	let yaml = load_yaml!("../args.yml");
	let matches = App::from_yaml(yaml).get_matches();
	
	let urls = match matches.values_of("URLS") {
	    Some(urls) => urls.collect(),
	    None => vec!["http://example.com"],
	};

	let urls = urls.into_iter().map(|url| url.parse().unwrap()).collect();

    
	// using the tokio runtime
	rt::run(fetch_urls(urls));
}

fn fetch_urls(urls: Vec<hyper::Uri>) -> impl Future<Item=(), Error = ()> {
    let client = Client::new();

    stream::iter_ok(urls)
        .for_each(move |url| fetch_url(url, &client))
}

fn fetch_url(url: hyper::Uri, client: &Client<HttpConnector>) -> impl Future<Item = (), Error = ()> {
    client
        .get(url)
        .and_then(|res| {
            println!("Status: {}", res.status());
            println!("Header: {:#?}", res.headers());

            res.into_body()
                .for_each(|chunk| {
                    io::stdout()
                    .write_all(&chunk)
                    .map_err(|_| panic!("no"))
                })
        })
        .map_err(|err| {
            println!("Error: {:?}", err);
        })
}
