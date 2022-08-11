use std::collections::HashMap;

use crate::types::MalType;

pub struct Env<'outer> {
	outer: Option<&'outer Env<'outer>>,
	data:  HashMap<String, MalType>,
}

impl<'outer> Env<'outer> {
	pub fn new<'outer_outer: 'outer>(
		outer: Option<&'outer_outer Env<'outer_outer>>,
	) -> Self {
		Self {
			outer,
			data: HashMap::new(),
		}
	}

	pub fn set(&mut self, key: String, value: MalType) {
		self.data.insert(key, value);
	}

	pub fn find<'ret, 'me: 'ret>(
		&'me self,
		key: impl AsRef<str>,
	) -> Option<&'ret Self> {
		if self.data.get(key.as_ref()).is_some() {
			Some(self)
		} else {
			self.outer.map(|outer| outer.find(key)).flatten()
		}
	}

	pub fn get(&self, key: impl AsRef<str>) -> Option<MalType> {
		self.data
			.get(key.as_ref())
			.cloned()
			.or_else(|| self.outer.map(|outer| outer.get(key)).flatten())
	}
}
