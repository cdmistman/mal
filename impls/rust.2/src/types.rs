use std::collections::HashMap;

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
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum MalHashKey {
	String(String),
	Keyword(String),
}
