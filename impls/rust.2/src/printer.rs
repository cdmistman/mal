use std::fmt::Display;

use crate::types::MalHashKey;
use crate::types::MalType;

/// Note that the `alternate` flag (`#`) is used to enable "print-readably"
impl Display for MalType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MalType::Bool(b) => write!(f, "{b}"),
			MalType::HashMap(vals) => {
				write!(
					f,
					"{{{}}}",
					list(
						&(vals.into_iter().fold(
							Vec::with_capacity(vals.len() * 2),
							|mut acc, (key, value)| {
								acc.push(match key {
									MalHashKey::Keyword(keyword) => {
										MalType::Keyword(keyword.clone())
									},
									MalHashKey::String(string) => {
										MalType::String(string.clone())
									},
								});
								acc.push(value.clone());
								acc
							}
						))
					)
				)
			},
			MalType::Keyword(keyword) => write!(f, ":{keyword}"),
			MalType::List(vals) => {
				write!(f, "({})", list(vals))
			},
			MalType::Nil => write!(f, "nil"),
			MalType::Number(num) => write!(f, "{num}"),
			MalType::String(string) if f.alternate() => {
				write!(f, "\"")?;
				for ch in string.chars() {
					match ch {
						'\n' => write!(f, "\\n"),
						'\\' => write!(f, "\\\\"),
						'"' => write!(f, "\\\""),
						ch => write!(f, "{ch}"),
					}?;
				}
				write!(f, "\"")
			},
			MalType::String(string) => {
				write!(f, "\"{}\"", string)
			},
			MalType::Symbol(symbol) => write!(f, "{symbol}"),
			MalType::Vector(vals) => {
				write!(f, "[{}]", list(vals))
			},

			MalType::NativeFn(_) => write!(f, "<native fn>"),
		}
	}
}

fn list(vals: &Vec<MalType>) -> String {
	vals.into_iter()
		.map(ToString::to_string)
		.collect::<Vec<_>>()
		.join(" ")
}
