use std::collections::HashMap;
use std::str::FromStr;

use eyre::Result;
use regex::Regex;

use crate::types::MalHashKey;
use crate::types::MalType;

#[derive(Clone)]
pub enum Token {}

#[derive(Clone)]
pub enum Ast {}

pub struct Reader<'source> {
	matches: Vec<&'source str>,
	posn:    usize,
}

impl<'source> Reader<'source> {
	pub fn new(input: &'source str) -> Self {
		Self {
			matches: tokenize(input),
			posn:    0,
		}
	}

	pub fn next(&mut self) -> Result<&'source str> {
		let res = self.peek()?;
		self.posn += 1;
		while let Ok(tok) = self.peek() && tok.starts_with(';') {
			self.posn += 1;
		}
		Ok(res)
	}

	pub fn peek(&self) -> Result<&'source str> {
		self.matches
			.get(self.posn)
			.cloned()
			.ok_or_else(|| eyre!("EOF"))
	}

	pub fn read_form(&mut self) -> Result<MalType> {
		match self.peek()? {
			"(" | "{" | "[" => self.read_list(),
			"'" => self.read_quote("quote"),
			"`" => self.read_quote("quasiquote"),
			"~" => self.read_quote("unquote"),
			"~@" => self.read_quote("splice-unquote"),
			"@" => self.read_quote("deref"),
			_ => self.read_atom(),
		}
	}

	fn read_quote(&mut self, kind: &str) -> Result<MalType> {
		let ("'" | "`" | "~" | "~@" | "@") = self.next()? else {
			return Err(eyre!("invalid quote"));
		};

		Ok(MalType::List(vec![
			MalType::Symbol(kind.to_string()),
			self.read_form()?,
		]))
	}

	fn read_list(&mut self) -> Result<MalType> {
		fn closes(l: &str, r: &str) -> bool {
			matches!((l, r), ("(", ")") | ("[", "]") | ("{", "}"))
		}

		let start @ ("(" | "[" | "{") = self.next()? else {
			return Err(eyre!("expected list"))
		};

		let mut list = Vec::new();
		loop {
			match self.peek()? {
				end @ (")" | "]" | "}") if closes(start, end) => {
					let _ = self.next(); // just guaranteed with peek
					break Ok(match start {
						"(" => MalType::List(list),
						"[" => MalType::Vector(list),
						"{" => MalType::HashMap({
							let mut map =
								HashMap::with_capacity(list.len() / 2);

							let mut items = list.into_iter();
							while let Some(key) = items.next() {
								let key = match key {
									MalType::Keyword(keyword) => {
										MalHashKey::Keyword(keyword)
									},
									MalType::String(string) => {
										MalHashKey::String(string)
									},
									_ => {
										return Err(eyre!(
											"invalid hashmap key"
										))
									},
								};
								let Some(value) = items.next() else {
									return Err(eyre!("no value for hashmap key"));
								};
								map.insert(key, value);
							}

							map
						}),
						_ => unreachable!(),
					});
				},
				")" | "]" | "}" => return Err(eyre!("improperly closed list")),
				_ => list.push(self.read_form()?),
			}
		}
	}

	fn read_atom(&mut self) -> Result<MalType> {
		Ok(match self.next()? {
			"false" => MalType::Bool(false),
			"nil" => MalType::Nil,
			"true" => MalType::Bool(true),
			keyword if keyword.starts_with(':') => {
				MalType::Keyword(keyword.chars().skip(1).collect())
			},
			string if string.starts_with('"') => {
				let mut chars = string.chars().skip(1);
				let mut string = String::with_capacity(string.len());
				let mut balanced = false;

				while let Some(ch) = chars.next() {
					if ch == '"' {
						balanced = true;
						break;
					}
					string.push(if '\\' == ch {
						// next value escaped
						match chars.next() {
							None => return Err(eyre!("EOF")),
							Some('n') => '\n',
							Some('\\') => '\\',
							Some('"') => '"',
							Some(ch) => {
								return Err(eyre!(
									"unexpected escape sequence: \\{ch} isn't \
									 a valid escape"
								))
							},
						}
					} else {
						// push to resulting string
						ch
					})
				}

				if !balanced {
					return Err(eyre!("unbalanced"));
				}
				MalType::String(string)
			},
			number_or_symbol => {
				if number_or_symbol
					.chars()
					.next()
					.map(char::is_numeric) // if the first char is a digit, it's a number
					.unwrap_or(false)
				{
					MalType::Number(f64::from_str(number_or_symbol)?)
				} else {
					MalType::Symbol(number_or_symbol.to_string())
				}
			},
		})
	}
}

pub fn read_str(input: &str) -> Result<MalType> {
	Reader::new(input).read_form()
}

pub fn tokenize(input: &str) -> Vec<&str> {
	lazy_static::lazy_static! {
		static ref REGEX: Regex = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#).unwrap();
	}

	REGEX
		.captures_iter(input)
		.map(|capture| {
			capture.get(1).map_or_else(
				|| panic!("somehow the capture didn't work"),
				|cap| cap.as_str(),
			)
		})
		.collect()
}
