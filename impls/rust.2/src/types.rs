use std::collections::HashMap;

use eyre::Result;

#[derive(Clone)]
pub enum MalType {
	Bool(bool),
	HashMap(HashMap<MalHashKey, MalType>),
	Keyword(String),
	List(Vec<MalType>),
	Nil,
	Number(f64),
	String(String),
	Symbol(String),
	Vector(Vec<MalType>),

	NativeFn(fn(&mut [MalType]) -> Result<MalType>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum MalHashKey {
	String(String),
	Keyword(String),
}

pub type Env = HashMap<String, MalType>;

impl MalType {
	pub fn eval(&mut self, env: &mut Env) -> Result<MalType> {
		match self {
			MalType::Symbol(sym) => env
				.get(sym)
				.cloned()
				.ok_or_else(|| eyre!("variable undefined: {sym}")),
			MalType::List(list) => list
				.into_iter()
				.map(|item| item.eval(env))
				.collect::<Result<Vec<_>>>()
				.map(MalType::List),
			val => Ok(val.clone()),
		}
	}
}
