#![feature(let_else)]

use std::collections::HashMap;
use std::io;
use std::io::Write;

use eyre::Result;
use rust2::builtin;
use rust2::env::Env;
use rust2::reader;
use rust2::types::MalType;

fn main() -> Result<()> {
	let mut stdin = io::stdin().lines();
	let mut stdout = io::stdout();

	let mut env = repl_env();

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

fn is_special_atom(ast: &MalType) -> bool {
	matches!(ast, MalType::Symbol(sym) if ["def!", "let*"].iter().any(|atom| sym == atom))
}

fn eval(ast: MalType, env: &mut Env<'_>) -> Result<MalType> {
	match ast {
		MalType::List(mut list)
			if list.get(0).map(is_special_atom).unwrap_or(false) =>
		{
			match list.remove(0) {
				MalType::Symbol(sym) if sym == "def!" => {
					let MalType::Symbol(key) = list.remove(0) else {
						return Err(eyre::eyre!("can't `def!` a non-variable"));
					};
					let value = eval(list.remove(0), env)?;
					env.set(key, value.clone());
					Ok(value)
				},
				MalType::Symbol(sym) if sym == "let*" => {
					let mut inner_env = Env::new(Some(env));
					let (MalType::List(bindings) | MalType::Vector(bindings)) = list.remove(0) else {
						return Err(eyre::eyre!("invalid `let*` form: expected bindings"));
					};
					let mut bindings = bindings.into_iter();
					while let Some(key) = bindings.next() {
						let MalType::Symbol(key) = key else {
							return Err(eyre::eyre!("invalid `let*` form: expected symbol for binding"));
						};
						let Some(value) = bindings.next() else {
							return Err(eyre::eyre!("invalid `let*` form: expected value to bind to variable `{key}`"));
						};
						let value = eval(value, &mut inner_env)?;
						inner_env.set(key, value)
					}

					eval(list.remove(0), &mut inner_env)
				},
				MalType::Symbol(special_atom) => {
					unreachable!("unrecognized special atom: {special_atom}")
				},
				_ => unreachable!("special atoms should be symbols"),
			}
		},
		MalType::List(list) if !list.is_empty() => {
			let MalType::List(mut list) = eval_ast(MalType::List(list), env)? else {
				unreachable!("expected `eval_ast` to return a list");
			};
			match list.remove(0) {
				MalType::NativeFn(fun) => fun(&mut list),
				_ => Err(eyre::eyre!("expected a function")),
			}
		},
		list @ MalType::List(_) => Ok(list),
		value => eval_ast(value, env),
	}
}

fn eval_ast(ast: MalType, env: &mut Env<'_>) -> Result<MalType> {
	match ast {
		MalType::Symbol(sym) => env
			.get(&sym)
			.ok_or_else(|| eyre::eyre!("'{sym}' not found")),
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

fn repl_env<'ret>() -> Env<'ret> {
	let mut env = Env::new(None);
	[
		("+".to_string(), MalType::NativeFn(builtin::add)),
		("-".to_string(), MalType::NativeFn(builtin::sub)),
		("*".to_string(), MalType::NativeFn(builtin::mul)),
		("/".to_string(), MalType::NativeFn(builtin::div)),
	]
	.into_iter()
	.for_each(|(key, value)| env.set(key, value));
	env
}