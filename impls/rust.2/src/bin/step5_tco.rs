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

	let env = rust2::core::ns();
	rep("(def! not (fn* (a) (if a false true)))", env.clone())?;

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
		match rep(&input, env.clone()) {
			Ok(result) => println!("{result}"),
			Err(err) => println!("ERROR: {err}"),
		}
	}
}

fn rep(input: &str, env: Env) -> Result<String> {
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

fn eval(mut ast: MalType, mut env: Env) -> Result<MalType> {
	'eval: loop {
		match ast {
			MalType::L(ListKind::List, mut list)
				if list.get(0).map(is_special_atom).unwrap_or(false) =>
			{
				match list.remove(0) {
					MalType::Symbol(sym) if sym == "def!" => {
						let MalType::Symbol(key) = list.remove(0) else {
							return Err(eyre::eyre!("can't `def!` a non-variable"));
						};
						let value = eval(list.remove(0), env.clone())?;
						env.set(key, value.clone());
						return Ok(value);
					},
					MalType::Symbol(sym) if sym == "do" => {
						let mut forms = list.into_iter().peekable();
						while let Some(form) = forms.next() {
							if forms.peek().is_some() {
								// there's another form after this, evaluate and
								// ignore ret
								eval(form, env.clone())?;
							} else {
								// this is the last value in the list, set it to
								// be evaluated and continue the eval loop
								ast = form;
								continue 'eval;
							}
						}

						// if we don't have any forms in this `do` (guaranteed
						// since i handle all other # of values above), just
						// return `nil`
						return Ok(MalType::Nil);
					},
					MalType::Symbol(sym) if sym == "fn*" => {
						let MalType::L(_, params) = list.remove(0) else {
							return Err(eyre::eyre!("invalid `fn*` form: expected parameter list"));
						};
						let params = params
							.into_iter()
							.map(|param_name| match param_name {
								MalType::Symbol(sym) => Ok(sym),
								_ => Err(eyre::eyre!(
									"invalid `fn*` form: expected symbol for \
									 parameter name"
								)),
							})
							.collect::<Result<Vec<_>>>()?;
						let body = list.remove(0);
						let closed = env.clone();

						return Ok(MalType::TCOFunction {
							ast:      Box::new(body.clone()),
							params:   params.clone(),
							env:      closed.clone(),
							function: Rc::new(move |args| {
								let closed_env = gen_env_from_param_app(
									Some(closed.clone()),
									params.clone().into_iter(),
									args.iter().map(Clone::clone),
								)?;
								eval(body.clone(), closed_env)
							}),
						});
					},
					MalType::Symbol(sym) if sym == "if" => {
						let cond = list.remove(0);
						let then = list.remove(0);
						let els = if list.is_empty() {
							MalType::Nil
						} else {
							list.remove(0)
						};
						ast = match eval(cond, env.clone())? {
							MalType::Bool(false) | MalType::Nil => els,
							_ => then,
						};
						continue 'eval;
					},
					MalType::Symbol(sym) if sym == "let*" => {
						let inner_env = Env::new(Some(env.clone()));
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
							let value = eval(value, inner_env.clone())?;
							inner_env.set(key, value)
						}

						ast = list.remove(0);
						env = inner_env;
						continue 'eval;
					},
					MalType::Symbol(special_atom) => {
						unreachable!(
							"unrecognized special atom: {special_atom}"
						)
					},
					_ => unreachable!("special atoms should be symbols"),
				}
			},
			MalType::L(ListKind::List, list) if !list.is_empty() => {
				let MalType::L(ListKind::List, mut list) = eval_ast(MalType::L(ListKind::List, list), env)? else {
					unreachable!("expected `eval_ast` to return a list");
				};
				return match list.remove(0) {
					MalType::Function(closure) => closure(&mut list),
					MalType::TCOFunction {
						ast: new_ast,
						params,
						env: closed_env,
						..
					} => {
						ast = *new_ast;
						env = gen_env_from_param_app(
							// evaluate using closed env
							Some(closed_env),
							params.clone().into_iter(),
							list.into_iter(),
						)?;
						continue 'eval;
					},
					not => Err(eyre::eyre!("expected a function (got {not})")),
				};
			},
			value => return eval_ast(value, env),
		}
	}
}

fn gen_env_from_param_app(
	parent_env: Option<Env>,
	mut param_names: impl Iterator<Item = String>,
	mut args: impl Iterator<Item = MalType> + Clone,
) -> Result<Env> {
	let mut params = Vec::with_capacity(param_names.size_hint().0);
	let mut va_bind = None;
	while let Some(name) = param_names.next() {
		if name == "&" {
			let Some(va_name) = param_names.next() else {
				return Err(eyre::eyre!("invalid `fn*` form: no variable binding for varargs"));
			};
			va_bind = Some(va_name);
		} else {
			params.push(name);
		}
	}

	let n_reg_args = params.len();
	let reg_args = Iterator::take(&mut args, n_reg_args);
	let res = Env::new_with_bindings(parent_env, params.into_iter(), reg_args);

	if let Some(va_bind) = va_bind {
		res.set(va_bind, MalType::L(ListKind::List, args.collect()));
	}

	Ok(res)
}

fn eval_ast(ast: MalType, env: Env) -> Result<MalType> {
	match ast {
		MalType::Symbol(sym) => env
			.get(&sym)
			.ok_or_else(|| eyre::eyre!("'{sym}' not found")),
		MalType::L(kind, list) => Ok(MalType::L(
			kind,
			list.into_iter()
				.map(|item| eval(item, env.clone()))
				.collect::<Result<Vec<_>>>()?,
		)),
		MalType::HashMap(map) => Ok(MalType::HashMap(
			map.into_iter()
				.map(|(key, value)| Ok((key, eval(value, env.clone())?)))
				.collect::<Result<HashMap<_, _>>>()?,
		)),
		ast => Ok(ast),
	}
}

fn print(input: MalType) -> String {
	format!("{input:#}")
}

#[cfg(test)]
mod tests {
	use std::iter;

	use super::*;

	#[test]
	fn test_gen_env_from_param_app() -> Result<()> {
		let basic = gen_env_from_param_app(None, iter::empty(), iter::empty())?;
		assert_eq!(basic, Env::new(None));

		let one_arg = gen_env_from_param_app(
			None,
			iter::once("foo".to_string()),
			iter::once(MalType::Nil),
		)?;
		assert_eq!(
			one_arg,
			Env::new_with_bindings_list(
				None,
				iter::once(("foo".to_string(), MalType::Nil))
			)
		);

		let multi_args = gen_env_from_param_app(
			None,
			vec!["foo".to_string(), "bar".to_string(), "quux".to_string()]
				.into_iter(),
			vec![MalType::Nil, MalType::Number(1.0), MalType::Bool(true)]
				.into_iter(),
		)?;
		assert_eq!(
			multi_args,
			Env::new_with_bindings_list(
				None,
				vec![
					("foo".to_string(), MalType::Nil),
					("bar".to_string(), MalType::Number(1.0)),
					("quux".to_string(), MalType::Bool(true))
				]
				.into_iter()
			)
		);

		let only_varargs = gen_env_from_param_app(
			None,
			vec!["&".to_string(), "va".to_string()].into_iter(),
			vec![MalType::Nil, MalType::Number(1.0), MalType::Bool(true)]
				.into_iter(),
		)?;
		assert_eq!(
			only_varargs,
			Env::new_with_bindings_list(
				None,
				vec![(
					"va".to_string(),
					MalType::L(ListKind::List, vec![
						MalType::Nil,
						MalType::Number(1.0),
						MalType::Bool(true)
					])
				)]
				.into_iter()
			)
		);

		let with_varargs = gen_env_from_param_app(
			None,
			vec![
				"foo".to_string(),
				"bar".to_string(),
				"&".to_string(),
				"va".to_string(),
			]
			.into_iter(),
			vec![
				MalType::Nil,
				MalType::Number(1.0),
				MalType::Bool(true),
				MalType::Bool(false),
				MalType::Keyword("cool".to_string()),
				MalType::String("hello, world!".to_string()),
			]
			.into_iter(),
		)?;
		assert_eq!(
			with_varargs,
			Env::new_with_bindings_list(
				None,
				vec![
					("foo".to_string(), MalType::Nil),
					("bar".to_string(), MalType::Number(1.0)),
					(
						"va".to_string(),
						MalType::L(ListKind::List, vec![
							MalType::Bool(true),
							MalType::Bool(false),
							MalType::Keyword("cool".to_string()),
							MalType::String("hello, world!".to_string()),
						])
					)
				]
				.into_iter()
			)
		);

		let empty_varargs = gen_env_from_param_app(
			None,
			vec!["foo".to_string(), "&".to_string(), "va".to_string()]
				.into_iter(),
			vec![MalType::Nil].into_iter(),
		)?;
		assert_eq!(
			empty_varargs,
			Env::new_with_bindings_list(
				None,
				vec![
					("foo".to_string(), MalType::Nil),
					("va".to_string(), MalType::L(ListKind::List, vec![]))
				]
				.into_iter()
			)
		);
		Ok(())
	}
}
