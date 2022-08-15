use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use eyre::Result;

#[derive(Clone)]
pub enum MalType {
	Bool(bool),
	HashMap(HashMap<MalHashKey, MalType>),
	Keyword(String),
	L(ListKind, Vec<MalType>),
	Nil,
	Number(f64),
	String(String),
	Symbol(String),
	Function(Rc<dyn Fn(&mut [MalType]) -> Result<MalType>>),
}

#[derive(Clone)]
pub enum ListKind {
	List,
	Vector,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MalHashKey {
	String(String),
	Keyword(String),
}

impl Debug for MalType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pr_str::<false>())
	}
}

impl PartialEq for MalType {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Bool(l), Self::Bool(r)) => l == r,
			(Self::HashMap(l), Self::HashMap(r)) => l == r,
			(Self::Keyword(l), Self::Keyword(r)) => l == r,
			(Self::L(_, l), Self::L(_, r)) => l == r,
			(Self::Nil, Self::Nil) => true,
			(Self::Number(l), Self::Number(r)) => l == r,
			(Self::String(l), Self::String(r)) => l == r,
			(Self::Symbol(l), Self::Symbol(r)) => l == r,
			(Self::Function(l), Self::Function(r)) => {
				Rc::as_ptr(l) == Rc::as_ptr(r)
			},
			_ => false,
		}
	}
}
