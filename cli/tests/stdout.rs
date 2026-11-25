use std::process::Command;

#[test]
fn debug_logs_go_to_stderr_not_stdout() {
	// An invalid syntax returns before any network access.
	let output = Command::new(env!("CARGO_BIN_EXE_check_if_email_exists"))
		.arg("not-an-email")
		.env("RUST_LOG", "debug")
		.output()
		.expect("failed to run the CLI");
	assert!(output.status.success());

	let stdout = String::from_utf8(output.stdout).unwrap();
	let json: serde_json::Value = serde_json::from_str(&stdout)
		.unwrap_or_else(|e| panic!("stdout is not valid JSON ({}):\n{}", e, stdout));
	assert_eq!(json["input"], "not-an-email");

	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("Checking email"), "stderr: {}", stderr);
}
