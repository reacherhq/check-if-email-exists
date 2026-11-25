use std::process::Command;

#[test]
fn debug_logs_go_to_stderr_not_stdout() {
	// An invalid syntax returns before any network access.
	let output = match Command::new(env!("CARGO_BIN_EXE_check_if_email_exists"))
		.arg("not-an-email")
		.env("RUST_LOG", "debug")
		.output()
	{
		Ok(output) => output,
		// ENOEXEC: cross-compiled targets run the tests under an emulator,
		// but the host kernel cannot execute the child binary directly.
		Err(e) if cfg!(unix) && e.raw_os_error() == Some(8) => return,
		Err(e) => panic!("failed to run the CLI: {}", e),
	};
	assert!(output.status.success());

	let stdout = String::from_utf8(output.stdout).unwrap();
	let json: serde_json::Value = serde_json::from_str(&stdout)
		.unwrap_or_else(|e| panic!("stdout is not valid JSON ({}):\n{}", e, stdout));
	assert_eq!(json["input"], "not-an-email");

	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("Checking email"), "stderr: {}", stderr);
}
