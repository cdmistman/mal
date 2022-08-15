use std::io::stdout;
use std::io::IoSlice;
use std::io::Write;
use std::rc::Rc;

use crate::env::Env;
use crate::types::ListKind;
use crate::types::MalType;

pub fn ns() -> Env {
	let bindings = [
		(
			"+",
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
			"-",
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
			"*",
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
			"/",
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
		(
			"list",
			MalType::Function(Rc::new(|args| {
				Ok(MalType::L(
					ListKind::List,
					args.into_iter().map(|arg| arg.clone()).collect(),
				))
			})),
		),
		(
			"list?",
			MalType::Function(Rc::new(|args| match args {
				[MalType::L(ListKind::List, ..), ..] => Ok(MalType::Bool(true)),
				_ => Ok(MalType::Bool(false)),
			})),
		),
		(
			"empty?",
			MalType::Function(Rc::new(|args| match args {
				[MalType::L(_, list), ..] => Ok(MalType::Bool(list.is_empty())),
				[MalType::HashMap(map), ..] => {
					Ok(MalType::Bool(map.is_empty()))
				},
				_ => Ok(MalType::Bool(false)),
			})),
		),
		(
			"count",
			MalType::Function(Rc::new(|args| match args {
				[MalType::L(_, list), ..] => {
					Ok(MalType::Number(list.len() as _))
				},
				[MalType::HashMap(map), ..] => {
					Ok(MalType::Number((map.len() / 2) as _))
				},
				_ => Ok(MalType::Number(0 as _)),
			})),
		),
		(
			"=",
			MalType::Function(Rc::new(|args| match args {
				[l, r] => Ok(MalType::Bool(l == r)),
				_ => Err(eyre!("`=` expects 2 parameters")),
			})),
		),
		(
			"<",
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Bool(l < r))
				},
				_ => Err(eyre!("`<` expects 2 numbers")),
			})),
		),
		(
			"<=",
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Bool(l <= r))
				},
				_ => Err(eyre!("`<=` expects 2 numbers")),
			})),
		),
		(
			">",
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Bool(l > r))
				},
				_ => Err(eyre!("`>` expects 2 numbers")),
			})),
		),
		(
			">=",
			MalType::Function(Rc::new(|args| match args {
				[MalType::Number(l), MalType::Number(r)] => {
					Ok(MalType::Bool(l >= r))
				},
				_ => Err(eyre!("`>=` expects 2 numbers")),
			})),
		),
		(
			"pr-str",
			MalType::Function(Rc::new(|args| {
				Ok(MalType::String(MalType::pr_list::<true>(
					args.iter(),
					"",
					"",
					" ",
				)))
			})),
		),
		(
			"str",
			MalType::Function(Rc::new(|args| {
				Ok(MalType::String(MalType::pr_list::<false>(
					args.iter(),
					"",
					"",
					"",
				)))
			})),
		),
		(
			"prn",
			MalType::Function(Rc::new(|args| {
				stdout().write_vectored(&[
					IoSlice::new(
						MalType::pr_list::<true>(args.iter(), "", "", " ")
							.as_bytes(),
					),
					IoSlice::new(b"\n"),
				])?;
				Ok(MalType::Nil)
			})),
		),
		(
			"println",
			MalType::Function(Rc::new(|args| {
				stdout().write_vectored(&[
					IoSlice::new(
						MalType::pr_list::<false>(args.iter(), "", "", " ")
							.as_bytes(),
					),
					IoSlice::new(b"\n"),
				])?;
				Ok(MalType::Nil)
			})),
		),
	]
	.into_iter()
	.map(|(sym, fun)| (sym.to_string(), fun));

	Env::new_with_bindings_list(None, bindings)
}
