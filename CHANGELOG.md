# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.7.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.7.0...v0.7.1) (2020-04-14)


### Features

* Add possibility to verify emails via proxy ([#286](https://github.com/amaurymartiny/check_if_email_exists/issues/286)) ([a0ab93f](https://github.com/amaurymartiny/check_if_email_exists/commit/a0ab93fde5105d594a8280b942d337ff76fbb517))

## [0.7.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.6.7...v0.7.0) (2020-03-26)


### ⚠ BREAKING CHANGES

* `email_exists` only takes one input now, an `EmailInput` which is built using the builder pattern.
```diff
- use check_if_email_exists::email_exists;
+ use check_if_email_exists::{email_exists, EmailInput};

- email_exists("someone@gmail.com", "user@example.org");
+ email_exists(
+   EmailInput::new("someone@gmail.com".to_string()).from_email("user@example.org".to_string())
+ )
```

`EmailInput` additionally takes a `hello_name()` method, which is used to set the name in the EHLO smtp command.

`--from` in CLI has been replaced with `--from-email`.

### Features

* Use builder pattern for EmailInput ([#254](https://github.com/amaurymartiny/check_if_email_exists/issues/254)) ([0c85d36](https://github.com/amaurymartiny/check_if_email_exists/commit/0c85d36cdccb37d8da9566f7e7baf5dbbd266740))

### [0.6.7](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.6.6...v0.6.7) (2020-03-20)

### [0.6.6](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.6.1...v0.6.6) (2020-03-01)


### Bug Fixes

* Allow http to listen to $PORT env variable ([#215](https://github.com/amaurymartiny/check_if_email_exists/issues/215)) ([3b0c262](https://github.com/amaurymartiny/check_if_email_exists/commit/3b0c262763bc9d52855ced90aa2a435a97d35d8b))

### [0.6.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.6.0...v0.6.1) (2020-02-18)


### Features

* Add --http-host flag to CLI ([#197](https://github.com/amaurymartiny/check_if_email_exists/issues/197)) ([55657b2](https://github.com/amaurymartiny/check_if_email_exists/commit/55657b251fcc22fad2ae53da4f62a017ff01d035))

## [0.6.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.5.0...v0.6.0) (2019-12-01)


### ⚠ BREAKING CHANGES

* - The `is_disposable` subfield has been moved from the `mx` field to a separate `misc` field

### Features

* Add a HTTP server behind the `--http` flag ([#85](https://github.com/amaurymartiny/check_if_email_exists/issues/85)) ([d8b733e](https://github.com/amaurymartiny/check_if_email_exists/commit/d8b733e5a571c512644b34219b5f2dfd9dc717b3))
* Add Dockerfile & `x86_64-unknown-linux-musl` target ([#86](https://github.com/amaurymartiny/check_if_email_exists/issues/86)) ([cba1241](https://github.com/amaurymartiny/check_if_email_exists/commit/cba124110be04d7febfeab68a6b825197b3aa1fb))

# [0.5.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.4.0...v0.5.0) (2019-11-16)


### Code Refactoring

* Use futures ([#78](https://github.com/amaurymartiny/check_if_email_exists/issues/78)) ([0e1f6b0](https://github.com/amaurymartiny/check_if_email_exists/commit/0e1f6b0))


### BREAKING CHANGES

* - The main function `email_exists` now returns a Future:
```rust
pub async fn email_exists(to_email: &str, from_email: &str) -> SingleEmail {}
```
- The `SmtpError::SmtpError` has been renamed to `SmtpError::LettreError` to show the underlying error more correctly (i.e., coming from `lettre` crate).
- The `BlockedByISP` error has been removed. Instead, you'll see e.g. `"connection refused"`, or whatever is returned by the SMTP server:
```json
{
  // ...,
  "smtp": {
    "error": {
      "type": "LettreError",
      "message": "connection refused"
    }
  },
}
```



# [0.4.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.3.2...v0.4.0) (2019-09-30)


### Features

* Add disposable email check ([#64](https://github.com/amaurymartiny/check_if_email_exists/issues/64)) ([1b2cea3](https://github.com/amaurymartiny/check_if_email_exists/commit/1b2cea3))


### BREAKING CHANGES

* the `smtp`'s object keys have changed. Instead of
```
{
  "deliverable": ...,
  "full_inbox": ...,
  "has_catch_all": ...
}
```
it now returns 
```
{
  "has_full_inbox": ...,
  "is_deliverable": ...,
  "is_disabled": ...,
  "is_catch_all": ...
}
```
where `is_disabled` checks if the address has been disabled/blocked by the email provider



## [0.3.2](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.3.1...v0.3.2) (2019-09-26)


### Bug Fixes

* **core:** SyntaxError also is type & message ([#60](https://github.com/amaurymartiny/check_if_email_exists/issues/60)) ([996633b](https://github.com/amaurymartiny/check_if_email_exists/commit/996633b))



## [0.3.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.3.0...v0.3.1) (2019-09-26)


### Bug Fixes

* Don't use virtual workspace, be able to build ([#59](https://github.com/amaurymartiny/check_if_email_exists/issues/59)) ([6c93893](https://github.com/amaurymartiny/check_if_email_exists/commit/6c93893))



# [0.3.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.2.3...v0.3.0) (2019-09-26)


### Features

* New error JSON format ([#56](https://github.com/amaurymartiny/check_if_email_exists/issues/56)) ([fec4315](https://github.com/amaurymartiny/check_if_email_exists/commit/fec4315))
* Output JSON information with CLI ([#53](https://github.com/amaurymartiny/check_if_email_exists/issues/53)) ([1d026d5](https://github.com/amaurymartiny/check_if_email_exists/commit/1d026d5))
* Return Result<EmailDetails> instead of Result<bool>, with much more details ([#23](https://github.com/amaurymartiny/check_if_email_exists/issues/23)) ([39b13f5](https://github.com/amaurymartiny/check_if_email_exists/commit/39b13f5))



## [0.2.3](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.2.2...v0.2.3) (2019-05-09)


### Bug Fixes

* Update version to correct version in cli ([992777c](https://github.com/amaurymartiny/check_if_email_exists/commit/992777c))



## [0.2.2](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.2.1...v0.2.2) (2019-05-09)


### Bug Fixes

* Fix travis and appveyor to build binaries ([f743e67](https://github.com/amaurymartiny/check_if_email_exists/commit/f743e67))



## [0.2.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.2.0...v0.2.1) (2019-05-09)


### Bug Fixes

* Refactor app to make travis build binaries ([#17](https://github.com/amaurymartiny/check_if_email_exists/issues/17)) ([9616ef5](https://github.com/amaurymartiny/check_if_email_exists/commit/9616ef5))



# [0.2.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.1.1...v0.2.0) (2019-05-09)


### Features

* Add serverless function ([#15](https://github.com/amaurymartiny/check_if_email_exists/issues/15)) ([532c4eb](https://github.com/amaurymartiny/check_if_email_exists/commit/532c4eb))
* Return Option<bool> instead of bool ([#13](https://github.com/amaurymartiny/check_if_email_exists/issues/13)) ([2aef345](https://github.com/amaurymartiny/check_if_email_exists/commit/2aef345))



## [0.1.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.1.0...v0.1.1) (2018-12-29)


# 0.1.0 (2018-12-29)


### Features

* Change codebase to Rust ([#7](https://github.com/amaurymartiny/check_if_email_exists/pull/7)) ([05569e4](https://github.com/amaurymartiny/check_if_email_exists/commit/05569e4900b4467fa6d7f03086343fac753fe4ad))
