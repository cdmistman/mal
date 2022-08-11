use std::collections::HashMap;

use debug_stub_derive::DebugStub;
use eyre::Result;

#[derive(Clone, DebugStub)]
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

	NativeFn(
		#[debug_stub = "<native fn>"] fn(&mut [MalType]) -> Result<MalType>,
	),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MalHashKey {
	String(String),
	Keyword(String),
}
