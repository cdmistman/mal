#![feature(let_else)]
/// This test is literally just so I can run the MAL test suite within cargo.

macro_rules! tests {
	[$($name:ident),* $(,)?] => {
		use eyre::ContextCompat;

		use std::env;
		use std::fs;
		use std::process::Command;

		$(
			#[test]
			fn $name() -> eyre::Result<()> {
				let pwd = env::current_dir()?;
				for entry in fs::read_dir(&pwd)? {
					let entry = entry?;
					if !entry.file_type()?.is_file() {
						continue;
					}
					let Ok(file_name) = entry.file_name().into_string() else {
						continue;
					};
					if file_name.starts_with(stringify!($name)) {
						fs::remove_file(file_name)?;
					}
				}

				let mal_dir = pwd
					.parent()
					.context("no impls/ folder")?
					.parent()
					.context("no mal/ folder")?;

				let output = Command::new("make")
					.arg(concat!("test^rust.2^", stringify!($name)))
					.current_dir(mal_dir)
					.output()?;

				let stderr = std::str::from_utf8(output.stderr.as_slice())?;
				let stdout = std::str::from_utf8(output.stdout.as_slice())?;
				println!("=== STDOUT\n{stdout}");

				if output.status.success() {
					Ok(())
				} else {
					let status = output.status;
					println!("===STDERR\n{stderr}");
					Err(eyre::eyre!("test failed with exit code {status}"))
				}
			}
		)*
	};
}

tests![step0, step1, step2, step3];
