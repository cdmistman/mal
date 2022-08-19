use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::types::MalType;

/// Note: this type is a shared reference *with interior mutability only*.
/// Mutability tracking only needs to occur internally, and instances can be
/// shared with `.clone()` safely.
#[derive(Clone)]
pub struct Env(Rc<RefCell<EnvInner>>);

#[derive(Debug)]
pub struct EnvInner {
	outer: Option<Env>,
	data:  HashMap<String, MalType>,
}

impl Env {
	fn raw_new(outer: Option<Env>, data: HashMap<String, MalType>) -> Self {
		Self(Rc::new(RefCell::new(EnvInner { outer, data })))
	}

	pub fn new(outer: Option<Env>) -> Self {
		Self::raw_new(outer, HashMap::new())
	}

	pub fn new_with_bindings(
		outer: Option<Env>,
		binds: impl Iterator<Item = String>,
		exprs: impl Iterator<Item = MalType>,
	) -> Self {
		Self::raw_new(outer, binds.zip(exprs).collect())
	}

	pub fn new_with_bindings_list(
		outer: Option<Env>,
		list: impl Iterator<Item = (String, MalType)>,
	) -> Self {
		Self::raw_new(outer, list.collect())
	}

	pub fn set(&self, key: String, value: MalType) {
		let mut inner = self.0.borrow_mut();
		inner.data.insert(key, value);
	}

	pub fn find(&self, key: impl AsRef<str>) -> Option<Env> {
		let inner = self.0.borrow();
		if inner.data.get(key.as_ref()).is_some() {
			Some(self.clone())
		} else {
			inner.outer.as_ref().map(|outer| outer.find(key)).flatten()
		}
	}

	pub fn get(&self, key: impl AsRef<str>) -> Option<MalType> {
		let inner = self.0.borrow();
		if let Some(res) = inner.data.get(key.as_ref()).cloned() {
			Some(res)
		} else {
			inner.outer.as_ref().map(|outer| outer.get(key)).flatten()
		}
	}
}

impl Debug for Env {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.borrow().fmt(f)
	}
}

impl PartialEq for Env {
	fn eq(&self, other: &Self) -> bool {
		self.0.borrow().data == other.0.borrow().data
	}
}

impl Eq for Env {}
