use std::fmt::Display;

use crate::types::ListKind;
use crate::types::MalHashKey;
use crate::types::MalType;

impl MalType {
	pub fn pr_str<const PRINT_READABLY: bool>(&self) -> String {
		match self {
			MalType::Bool(b) => format!("{b}"),
			MalType::Function(_) => "#<function>".to_string(),
			MalType::Keyword(kw) => format!(":{kw}"),
			MalType::Nil => "nil".to_string(),
			MalType::Number(num) => format!("{num}"),
			MalType::String(string) if PRINT_READABLY => {
				string
					.chars()
					.fold("\"".to_string(), |mut res, ch| match ch {
						'\n' => res + "\\n",
						'"' => res + "\\\"",
						'\\' => res + "\\\\",
						ch => {
							res.push(ch);
							res
						},
					}) + "\""
			},
			MalType::String(string) => string.clone(),
			MalType::Symbol(sym) => sym.clone(),

			MalType::HashMap(vals) => {
				return Self::pr_list::<PRINT_READABLY>(
					vals.iter()
						.map(|(key, val)| {
							[
								match key {
									MalHashKey::String(string) => {
										MalType::String(string.clone())
									},
									MalHashKey::Keyword(kw) => {
										MalType::Keyword(kw.clone())
									},
								},
								val.clone(),
							]
						})
						.flatten()
						.collect::<Vec<_>>()
						.as_slice()
						.into_iter(),
					"{",
					"}",
					" ",
				)
			},
			MalType::L(ListKind::List, list) => {
				Self::pr_list::<PRINT_READABLY>(list.iter(), "(", ")", " ")
			},
			MalType::L(ListKind::Vector, list) => {
				Self::pr_list::<PRINT_READABLY>(list.iter(), "[", "]", " ")
			},
		}
	}

	pub fn pr_list<const PRINT_READABLY: bool>(
		list: impl Iterator<Item = &'_ Self>,
		start: &str,
		end: &str,
		join: &str,
	) -> String {
		format!(
			"{start}{}{end}",
			list.map(Self::pr_str::<PRINT_READABLY>)
				.collect::<Vec<_>>()
				.join(join)
		)
	}
}

/// Note that the `alternate` flag (`#`) is used to enable "print-readably"
impl Display for MalType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		return write!(
			f,
			"{}",
			if f.alternate() {
				self.pr_str::<true>()
			} else {
				self.pr_str::<false>()
			}
		);
	}
}
