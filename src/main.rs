#![allow(unused, clippy::all)]
#![feature(let_chains, try_blocks)]

/*
	other useful crates:
	cargo add derive_more --features full # derive macros for more traits
	cargo add variantly # introspection for enum variants
	cargo add validator # validation library
	cargo add bon # builder pattern
	cargo add strum strum_macros # set of macros and traits for working with enums and strings
	cargo add nestruct # nested structs
	cargo add reqwest # http client
	cargo add itertools # iterators
*/

use anyhow::Result;
use thin_logger::log;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok(); // load .env file
	thin_logger::build(None).init(); // init logging

	log::info!("Hello, world!");

	Ok(())
}
