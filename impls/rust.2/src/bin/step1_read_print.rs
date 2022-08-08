#![feature(let_else)]

use std::io;
use std::io::Write;

use eyre::Result;
use rust2::reader;
use rust2::types::MalType;

fn main() -> Result<()> {
	let mut stdin = io::stdin().lines();
	let mut stdout = io::stdout();

	loop {
		print!("user> ");
		stdout.flush()?;

		let Some(input) = stdin.next() else {
			eprintln!("Goodbye.");
			return Ok(());
		};
		let input = match input {
			Ok(input) => input,
			Err(err) => {
				eprintln!("Error reading from stdin: {}", err);
				continue;
			},
		};
		match rep(&input) {
			Ok(result) => println!("{result}"),
			Err(err) => println!("ERROR: {err}"),
		}
	}
}

fn rep(input: &str) -> Result<String> {
	let res_read = read(input)?;
	let res_eval = eval(res_read);
	Ok(print(res_eval))
}

fn read(input: &str) -> Result<MalType> {
	reader::read_str(input)
}

fn eval(input: MalType) -> MalType {
	input
}

fn print(input: MalType) -> String {
	format!("{input:#}")
}
