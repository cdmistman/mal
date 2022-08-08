/// This test is literally just so I can run the MAL test suite within cargo.

macro_rules! tests {
	[$($name:ident),* $(,)?] => {
		use eyre::ContextCompat;

		use std::env;
		use std::process::Command;

		$(
			#[test]
			fn $name() -> eyre::Result<()> {
				let pwd = env::current_dir()?;
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
				println!("=== STDOUT\n{stdout}\n\n===STDERR\n{stderr}");

				if output.status.success() {
					Ok(())
				} else {
					let status = output.status;
					Err(eyre::eyre!("test failed with exit code {status}"))
				}
			}
		)*
	};
}

tests![step0,];
