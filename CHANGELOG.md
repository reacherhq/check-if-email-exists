# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.8.6](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.5...v0.8.6) (2020-06-28)


### Bug Fixes

* Add additional error check for undeliverable ([#374](https://github.com/amaurymartiny/check-if-email-exists/issues/374)) ([e52a8f0](https://github.com/amaurymartiny/check-if-email-exists/commit/e52a8f0941fd53c9b087e6e59a7018d11af72dff))
* Use HTTP requests to verify Yahoo emails ([#412](https://github.com/amaurymartiny/check-if-email-exists/issues/412)) ([5fad57d](https://github.com/amaurymartiny/check-if-email-exists/commit/5fad57d88ef92d65c7d493cdcb45eff347d6a286))

### [0.8.5](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.8.4...v0.8.5) (2020-05-21)


### Features

* Expose misc, syntax, mx, smtp modules ([#373](https://github.com/amaurymartiny/check_if_email_exists/issues/373)) ([7c1d741](https://github.com/amaurymartiny/check_if_email_exists/commit/7c1d741f00b3a807b190140a1d91a42bce29470c))

### [0.8.4](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.8.3...v0.8.4) (2020-05-19)


### Bug Fixes

* Fix is_reachable with wrong syntax ([#352](https://github.com/amaurymartiny/check_if_email_exists/issues/352)) ([b0f0209](https://github.com/amaurymartiny/check_if_email_exists/commit/b0f02090edc0bb8947ab826415fa3bf8b5db55f0))

### [0.8.3](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.8.2...v0.8.3) (2020-05-12)


### Bug Fixes

* Lowercase Reachable enum variants ([#351](https://github.com/amaurymartiny/check_if_email_exists/issues/351)) ([b88c20e](https://github.com/amaurymartiny/check_if_email_exists/commit/b88c20ef5bc947ecd8cc646a9e6c583df0bef4d7))

### [0.8.2](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.8.1...v0.8.2) (2020-05-12)


### Features

* Add `is_reachable` top field ([#350](https://github.com/amaurymartiny/check_if_email_exists/issues/350)) ([e7abb17](https://github.com/amaurymartiny/check_if_email_exists/commit/e7abb17ef29610fbe9210f42830c0ba02bb130b7))
* Detect role-based accounts ([#348](https://github.com/amaurymartiny/check_if_email_exists/issues/348)) ([7c612fd](https://github.com/amaurymartiny/check_if_email_exists/commit/7c612fda110729ece094d0b022db05fa4e6b27b4))


### Bug Fixes

* Add "Unknown user" smtp error check ([#347](https://github.com/amaurymartiny/check_if_email_exists/issues/347)) ([47eb578](https://github.com/amaurymartiny/check_if_email_exists/commit/47eb5780f692f54aadf264b107996bb2d1a31a56))
* Add more error strings matching ([#323](https://github.com/amaurymartiny/check_if_email_exists/issues/323)) ([f5392d4](https://github.com/amaurymartiny/check_if_email_exists/commit/f5392d4befcee6e4d935e1585066eae3b57ec6fa))

### [0.8.1](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.8.0...v0.8.1) (2020-05-09)


### Bug Fixes

* Lowercase the error string before matching ([#321](https://github.com/amaurymartiny/check_if_email_exists/issues/321)) ([d983b2f](https://github.com/amaurymartiny/check_if_email_exists/commit/d983b2fe960ed46c4bd03c55b39d7ea58be5124f))

## [0.8.0](https://github.com/amaurymartiny/check_if_email_exists/compare/v0.7.1...v0.8.0) (2020-05-08)


### ⚠ BREAKING CHANGES

* This new version includes an overhaul of the codebase, mainly to prepare the groundwork for the upcoming work on bulk validation. These changes include:

- The main function `email_exists` has been renamed to `check_email`:

```diff
- email_exists(&input).await;
+ check_email(&input).await;
```

- The input `EmailInput` has been renamed to `CheckEmailInput`. Its `::new()` method, instead of taking a single `String`, now takes `Vec<String>`.

- The output `SingleEmail` has been renamed to `CheckEmailOutput`. The main function `check_emails` now returns a `Vec<CheckEmailOutput>`.

```rust
pub async fn check_email(inputs: &CheckEmailInput) -> Vec<CheckEmailOutput>
```

- The `syntax` field in `CheckEmailOutput` is no longer a `Result<SyntaxDetails, SyntaxError>`, but only `SyntaxDetails`. Error cases are guaranteed not to happen for syntax validation.

- The `misc`, `mx`, and `smtp` fields' signatures stay the same: `Result<{Misc,Mx,Smtp}Details, {Misc,Mx,Smtp}Error>`. However, the `Result` is an `Err` only when an internal error arrives. In case of errors due to user input (e.g. incorrect email inputted), the `Default` trait has been implemented on `{Misc,Mx,Smtp}Details` and will be returned. As such, the `Skipped` variant of error enums has been removed.

```diff
{
  "input": "foo@bar.baz",
  "mx": {
-    "error": { "cannot resolve" }
+    "accepts_mail": false, // This is Default
+    "records": [] // This is Default
  }
```

- The `misc`, `mx`, `smtp`, `syntax` modules have been made private.
* The field `syntax.valid_format` has been renamed to `syntax.is_valid_syntax`.

### Bug Fixes

* Rename valid_format to is_valid_syntax ([#288](https://github.com/amaurymartiny/check_if_email_exists/issues/288)) ([eae6482](https://github.com/amaurymartiny/check_if_email_exists/commit/eae64821c31d0193f77d9137ec4e7d6131f91ccb))


* Rename main function to `check_email` ([#319](https://github.com/amaurymartiny/check_if_email_exists/issues/319)) ([bd12b7d](https://github.com/amaurymartiny/check_if_email_exists/commit/bd12b7dbbd6c090fcdf80e3d6bbe475cd1d82b9a))

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
