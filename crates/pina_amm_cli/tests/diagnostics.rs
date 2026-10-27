//! Process-level checks for safe failure output.

use std::process::Command;

#[test]
fn text_errors_escape_terminal_controls() {
	let output = Command::new(env!("CARGO_BIN_EXE_pina-amm"))
		.args([
			"--url",
			"bad\u{1b}[2J\u{1b}]52;c;dGVzdA==\u{7}",
			"config",
			"show",
			"--index",
			"0",
		])
		.output()
		.expect("run CLI");
	let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");

	assert_eq!(output.status.code(), Some(1));
	assert_eq!(output.stdout.as_slice(), b"");
	assert!(!stderr.contains(['\u{1b}', '\u{7}']));
	assert!(stderr.contains("bad\\u{1b}[2J\\u{1b}]52;c;dGVzdA==\\u{7}"));
}

#[test]
fn json_errors_are_one_safe_json_object_on_stderr() {
	let output = Command::new(env!("CARGO_BIN_EXE_pina-amm"))
		.args([
			"--json",
			"--url",
			"bad\u{1b}[2J\u{9b}2J",
			"config",
			"show",
			"--index",
			"0",
		])
		.output()
		.expect("run CLI");
	let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");

	assert_eq!(output.status.code(), Some(1));
	assert_eq!(output.stdout.as_slice(), b"");
	assert_eq!(stderr.lines().count(), 1);
	assert!(!stderr.contains(['\u{1b}', '\u{9b}']));

	let value: serde_json::Value = serde_json::from_str(&stderr).expect("JSON error object");
	let message = value["error"].as_str().expect("error message");

	assert!(message.contains("bad\\u{1b}[2J\\u{9b}2J"));
}
