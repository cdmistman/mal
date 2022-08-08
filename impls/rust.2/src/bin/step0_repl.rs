use std::io;
use std::io::Write;

fn main() -> io::Result<()> {
	let mut stdin = io::stdin().lines();
	let mut stdout = io::stdout();

	loop {
		print!("user> ");
		stdout.flush()?;

		let input = match stdin.next() {
			Some(input) => input?,
			None => {
				println!("");
				continue;
			},
		};
		let result = rep(input);
		println!("{result}");
	}
}

fn rep(input: String) -> String {
	let res_read = read(input);
	let res_eval = eval(res_read);
	print(res_eval)
}

fn read(input: String) -> String {
	input
}

fn eval(input: String) -> String {
	input
}

fn print(input: String) -> String {
	input
}
