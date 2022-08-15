#![feature(let_else)]

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::rc::Rc;

use eyre::eyre;
use eyre::Result;
use rust2::reader;
use rust2::types::ListKind;
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

fn eval(ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast {
		MalType::L(ListKind::List, list) if !list.is_empty() => {
			let MalType::L(ListKind::List, mut list) = eval_ast(MalType::L(ListKind::List, list), env)? else {
				return Err(eyre::eyre!("expected `eval_ast` to return a list (this should not happen)"));
			};
			let fun = list.remove(0);
			match fun {
				MalType::Function(fun) => fun(&mut list),
				_ => Err(eyre::eyre!("expected a function")),
			}
		},
		value => eval_ast(value, env),
	}
}

fn eval_ast(ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast {
		MalType::Symbol(sym) => env
			.get(&sym)
			.cloned()
			.ok_or_else(|| eyre::eyre!("Symbol not found: {sym}")),
		MalType::L(kind, list) => Ok(MalType::L(
			kind,
			list.into_iter()
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
		(
			"+".to_string(),
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Number(*l + *r))
				},
				[_, _] => Err(eyre!("`+` expects 2 numbers")),
				args => Err(eyre!(
					"`+` expects 2 args: {} were provided",
					args.len()
				)),
			})),
		),
		(
			"-".to_string(),
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Number(*l - *r))
				},
				[_, _] => Err(eyre!("`-` expects 2 numbers")),
				args => Err(eyre!(
					"`-` expects 2 args: {} were provided",
					args.len()
				)),
			})),
		),
		(
			"*".to_string(),
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Number(*l * *r))
				},
				[_, _] => Err(eyre!("`*` expects 2 numbers")),
				args => Err(eyre!(
					"`*` expects 2 args: {} were provided",
					args.len()
				)),
			})),
		),
		(
			"/".to_string(),
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Number(*l / *r))
				},
				[_, _] => Err(eyre!("`/` expects 2 numbers")),
				args => Err(eyre!(
					"`/` expects 2 args: {} were provided",
					args.len()
				)),
			})),
		),
	])
}
