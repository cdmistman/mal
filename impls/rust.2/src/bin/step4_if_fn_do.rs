#![feature(let_else)]

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::rc::Rc;

use eyre::Result;
use rust2::env::Env;
use rust2::reader;
use rust2::types::ListKind;
use rust2::types::MalType;

fn main() -> Result<()> {
	let mut stdin = io::stdin().lines();
	let mut stdout = io::stdout();

	let mut env = rust2::core::ns();
	rep("(def! not (fn* (a) (if a false true)))", &mut env)?;

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
	matches!(ast, MalType::Symbol(sym) if ["def!", "do", "fn*", "if", "let*"].iter().any(|atom| sym == atom))
}

fn eval(ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast {
		MalType::L(ListKind::List, mut list)
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
				MalType::Symbol(sym) if sym == "do" => list
					.into_iter()
					.fold(Ok(MalType::Nil), |_, expr| eval(expr, env)),
				MalType::Symbol(sym) if sym == "fn*" => {
					let closed = env.clone();
					let MalType::L(_, param_names) = list.remove(0) else {
						return Err(eyre::eyre!("invalid `fn*` form: expected parameter list"));
					};
					let mut param_names = param_names.into_iter();
					let body = list.remove(0);

					let mut params = Vec::with_capacity(param_names.len());
					let mut va_bind = None;
					while let Some(name) = param_names.next() {
						match name {
							MalType::Symbol(var_arg) if var_arg == "&" => {
								param_names
									.next()
									.map(|va| match va {
										MalType::Symbol(sym) => {
											Some(va_bind = Some(sym))
										},
										_ => None,
									})
									.flatten()
									.ok_or_else(|| {
										eyre::eyre!(
											"invalid `fn*` form: no variable \
											 binding for varargs"
										)
									})?
							},
							MalType::Symbol(name) => params.push(name),
							_ => {
								return Err(eyre::eyre!(
									"invalid `fn*` form: parameters must be \
									 symbols"
								))
							},
						}
					}
					Ok(MalType::Function(Rc::new(move |args| {
						let mut va_bind = va_bind.clone();

						let closed_env = closed.clone();
						let params = params.clone();
						let va_start = va_bind
							.as_ref()
							.map(|_| params.len())
							.unwrap_or_else(|| args.len());
						let arg_binds =
							args[..va_start].into_iter().map(|arg| arg.clone());

						let mut app_env = Env::new_with_bindings(
							Some(closed_env),
							params.into_iter(),
							arg_binds,
						);

						va_bind.take().map(|bind| {
							app_env.set(
								bind,
								MalType::L(
									ListKind::List,
									args[va_start..]
										.into_iter()
										.map(|arg| arg.clone())
										.collect(),
								),
							)
						});

						eval(body.clone(), &mut app_env)
					})))
				},
				MalType::Symbol(sym) if sym == "if" => {
					let cond = list.remove(0);
					let then = list.remove(0);
					let els = if list.is_empty() {
						MalType::Nil
					} else {
						list.remove(0)
					};
					match eval(cond, env)? {
						MalType::Bool(false) | MalType::Nil => eval(els, env),
						_ => eval(then, env),
					}
				},
				MalType::Symbol(sym) if sym == "let*" => {
					let mut inner_env = Env::new(Some(env.clone()));
					let MalType::L(_, bindings) = list.remove(0) else {
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
		MalType::L(ListKind::List, list) if !list.is_empty() => {
			let MalType::L(ListKind::List, mut list) = eval_ast(MalType::L(ListKind::List, list), env)? else {
				unreachable!("expected `eval_ast` to return a list");
			};
			match list.remove(0) {
				MalType::Function(closure) => closure(&mut list),
				not => Err(eyre::eyre!("expected a function (got {not})")),
			}
		},
		value => eval_ast(value, env),
	}
}

fn eval_ast(ast: MalType, env: &mut Env) -> Result<MalType> {
	match ast {
		MalType::Symbol(sym) => env
			.get(&sym)
			.ok_or_else(|| eyre::eyre!("'{sym}' not found")),
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
