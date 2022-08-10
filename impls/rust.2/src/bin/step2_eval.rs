#![feature(let_else)]

use std::collections::HashMap;
use std::io;
use std::io::Write;

use eyre::Result;
use rust2::builtin;
use rust2::reader;
use rust2::types::MalType;

type Env = HashMap<String, MalType>;

fn main() -> Result<()> {
	let mut stdin = io::stdin().lines();
	let mut stdout = io::stdout();

	let mut env = std_env();

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
		match rep(&input, &mut env) {
			Ok(result) => println!("{result}"),
			Err(err) => println!("ERROR: {err}"),
		}
	}
}

fn rep(input: &str, env: &mut Env) -> Result<String> {
	let res_read = read(input)?;
	let res_eval = eval(res_read, env)?;
	Ok(print(res_eval))
}

fn read(input: &str) -> Result<MalType> {
	reader::read_str(input)
}

fn eval(mut ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast.eval(env)? {
		MalType::List(list) if !list.is_empty() => {
			let MalType::List(mut list) = eval_ast(MalType::List(list), env)? else {
				return Err(eyre::eyre!("expected `eval_ast` to return a list (this should not happen)"));
			};
			let fun = list.remove(0);
			match fun {
				MalType::NativeFn(fun) => fun(&mut list),
				_ => Err(eyre::eyre!("expected a function")),
			}
		},
		list @ MalType::List(_) => Ok(list),
		value => eval_ast(value, env),
	}
}

fn eval_ast(ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast {
		MalType::Symbol(sym) => env
			.get(&sym)
			.cloned()
			.ok_or_else(|| eyre::eyre!("Symbol not found: {sym}")),
		MalType::List(list) => Ok(MalType::List(
			list.into_iter()
				.map(|item| eval(item, env))
				.collect::<Result<Vec<_>>>()?,
		)),
		MalType::Vector(vector) => Ok(MalType::Vector(
			vector
				.into_iter()
				.map(|item| eval(item, env))
				.collect::<Result<Vec<_>>>()?,
		)),
		MalType::HashMap(map) => Ok(MalType::HashMap(
			map.into_iter()
				.map(|(key, value)| Ok((key, eval(value, env)?)))
				.collect::<Result<HashMap<_, _>>>()?,
		)),
		ast => Ok(ast),
	}
}

fn print(input: MalType) -> String {
	format!("{input:#}")
}

fn std_env() -> Env {
	Env::from([
		("+".to_string(), MalType::NativeFn(builtin::add)),
		("-".to_string(), MalType::NativeFn(builtin::sub)),
		("*".to_string(), MalType::NativeFn(builtin::mul)),
		("/".to_string(), MalType::NativeFn(builtin::div)),
	])
}
