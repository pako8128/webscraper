#[macro_use]
extern crate clap;
extern crate hyper;

use clap::App;

use hyper::Client;
use hyper::rt::{self, Future, Stream}; // hyper re-exposes the traits from future crate

use std::io::prelude::*;
use std::io;
use std::fs::File;

fn main() {
	// parse the command line arguments
	let yaml = load_yaml!("../args.yml");
	let matches = App::from_yaml(yaml).get_matches();
	
	let url = match matches.value_of("URL") {
	    Some(url) => url,
	    None => "http://example.com",
	};

	let url = url.parse().unwrap();
	
	// using the tokio runtime
	rt::run(fetch_url(url));
}

 
fn fetch_url(url: hyper::Uri) -> impl Future<Item=(), Error=()> {
    let client = Client::new();

    client
        .get(url)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: {:#?}", res.headers());

            res.into_body().for_each(|chunk| {
                io::stdout().write_all(&chunk)
                    .map_err(|e| panic!("example expects stdout is open, error={}", e))
            })
        })
        .map(|_| {
            println!("\n\nDone.");
        })
        .map_err(|err| {
            eprintln!("Error {}", err);
        })
}
