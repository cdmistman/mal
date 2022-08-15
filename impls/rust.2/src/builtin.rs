use eyre::Result;

use crate::types::MalType;

pub fn add(args: &mut [MalType]) -> Result<MalType> {
	match args {
		[MalType::Number(l), MalType::Number(r)] => {
			Ok(MalType::Number(*l + *r))
		},
		[_, _] => Err(eyre!("`+` expects 2 numbers")),
		args => Err(eyre!("`+` expects 2 args: {} were provided", args.len())),
	}
}

pub fn sub(args: &mut [MalType]) -> Result<MalType> {
	match args {
		[MalType::Number(l), MalType::Number(r)] => {
			Ok(MalType::Number(*l - *r))
		},
		[_, _] => Err(eyre!("`+` expects 2 numbers")),
		args => Err(eyre!("`+` expects 2 args: {} were provided", args.len())),
	}
}

pub fn mul(args: &mut [MalType]) -> Result<MalType> {
	match args {
		[MalType::Number(l), MalType::Number(r)] => {
			Ok(MalType::Number(*l * *r))
		},
		[_, _] => Err(eyre!("`+` expects 2 numbers")),
		args => Err(eyre!("`+` expects 2 args: {} were provided", args.len())),
	}
}

pub fn div(args: &mut [MalType]) -> Result<MalType> {
	match args {
		[MalType::Number(l), MalType::Number(r)] => {
			Ok(MalType::Number(*l / *r))
		},
		[_, _] => Err(eyre!("`+` expects 2 numbers")),
		args => Err(eyre!("`+` expects 2 args: {} were provided", args.len())),
	}
}
