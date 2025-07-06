# Changelog

All notable changes to this project will be documented in this file. The changes in this project follow [Convention Commits](https://www.conventionalcommits.org/en/v1.0.0/).

# [](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.6...v) (2025-07-06)



## [0.11.6](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.5...v0.11.6) (2025-07-06)


### Bug Fixes

* Bring back `{yahoo,hotmailb2c}_verif_method` ([#1606](https://github.com/reacherhq/check-if-email-exists/issues/1606)) ([3fbe520](https://github.com/reacherhq/check-if-email-exists/commit/3fbe5200a3d8608fbd72c0f2a5917326c1f8ec91))
* Fix rabbitmq docker compose ([7c3856e](https://github.com/reacherhq/check-if-email-exists/commit/7c3856ebec6089b37b3dd30e3c4f13df9fb4e73a))



## [0.11.5](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.4...v0.11.5) (2025-04-29)


### Features

* Add optional timeout on proxy (env var: `RCH__PROXY__TIMEOUT_MS`) ([#1595](https://github.com/reacherhq/check-if-email-exists/issues/1595)) ([0e51eb6](https://github.com/reacherhq/check-if-email-exists/commit/0e51eb686dad6bd2ec827e785bf9c30ccc88cde1))



## [0.11.4](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.3...v0.11.4) (2025-04-28)


### Bug Fixes

* Add "utilisateur inconnu" in invalid parser ([#1594](https://github.com/reacherhq/check-if-email-exists/issues/1594)) ([fb91653](https://github.com/reacherhq/check-if-email-exists/commit/fb9165303e2d7be59ed2fa4f0682e8592bc0c5e7))



## [0.11.3](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.2...v0.11.3) (2025-03-29)


### Bug Fixes

* Fix version in logs ([fa6be78](https://github.com/reacherhq/check-if-email-exists/commit/fa6be7867abae981b0d82fde24e0310b9759ab1f))



## [0.11.2](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.1...v0.11.2) (2025-03-28)


### Bug Fixes

* Remove max requests per minute/day ([07a6d96](https://github.com/reacherhq/check-if-email-exists/commit/07a6d96416f52ac0824e7e7ac665fd2169ddc7ec))
* Show thread ID in logs ([#1579](https://github.com/reacherhq/check-if-email-exists/issues/1579)) ([3388163](https://github.com/reacherhq/check-if-email-exists/commit/3388163d03b66ba92455be8404441e8555a9d53c))


### Reverts

* "Show thread ID in logs ([#1579](https://github.com/reacherhq/check-if-email-exists/issues/1579))" ([56e7838](https://github.com/reacherhq/check-if-email-exists/commit/56e7838f28067b05b58f1fcd166368a915aafbbc))



## [0.11.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.11.0...v0.11.1) (2025-03-24)


### Bug Fixes

* Revert back to using lowest-priority MX record ([#1578](https://github.com/reacherhq/check-if-email-exists/issues/1578)) ([60468b3](https://github.com/reacherhq/check-if-email-exists/commit/60468b3f533491a0dff6a42e7096f34ece19896c))



# [0.11.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.10.1...v0.11.0) (2025-02-19)


* feat!: Allow multiple proxies (#1562) ([eed5a15](https://github.com/reacherhq/check-if-email-exists/commit/eed5a1536af37877f12eebab6481acaa6efa55c5)), closes [#1562](https://github.com/reacherhq/check-if-email-exists/issues/1562)


### Bug Fixes

* **docker:** Fix dockerfile entrypoint ([d1d3326](https://github.com/reacherhq/check-if-email-exists/commit/d1d3326af88a85b2192796d8d2c92ff854b5644d))
* Don't show proxy full info in logs ([2668ce1](https://github.com/reacherhq/check-if-email-exists/commit/2668ce14418076b00f36f18a370070ac1f3754bf))
* Fix AWS login in Action ([6dd6fb0](https://github.com/reacherhq/check-if-email-exists/commit/6dd6fb02b77049d2a9fc2510ed438b1ac8ab60aa))
* Fix correct Dockerfile in Action ([4d4d91e](https://github.com/reacherhq/check-if-email-exists/commit/4d4d91ea7b43678d472e4f7f6ae6952625b2f478))
* Fixed inverted hello-name and from-email in CLI ([#1565](https://github.com/reacherhq/check-if-email-exists/issues/1565)) ([a53561e](https://github.com/reacherhq/check-if-email-exists/commit/a53561e087593ccc887b45943f54855b9cc6ae85))
* Improve logging, add retries for Yahoo headless, switch to rustls ([#1549](https://github.com/reacherhq/check-if-email-exists/issues/1549)) ([b1377db](https://github.com/reacherhq/check-if-email-exists/commit/b1377db2b32155d766a09a76864fc9b0990833e6))
* Make new config backwards-compatible ([#1567](https://github.com/reacherhq/check-if-email-exists/issues/1567)) ([b824e2c](https://github.com/reacherhq/check-if-email-exists/commit/b824e2c988ee4eef021b97fc65ebcfa36a166d7f))
* Reinstate proxy in JSON request ([#1569](https://github.com/reacherhq/check-if-email-exists/issues/1569)) ([c36e6e0](https://github.com/reacherhq/check-if-email-exists/commit/c36e6e09c9079de210d288b84d79b984e2ea77f0))


### Features

* Add `misc.is_b2c` field ([#1553](https://github.com/reacherhq/check-if-email-exists/issues/1553)) ([14a6759](https://github.com/reacherhq/check-if-email-exists/commit/14a6759d805d2051a4a1e1d81588279cb9c85336))
* Add AWS SQS support ([#1554](https://github.com/reacherhq/check-if-email-exists/issues/1554)) ([92be54e](https://github.com/reacherhq/check-if-email-exists/commit/92be54ebfe4a2d19101141f55e94fc8e9588ff95))


### BREAKING CHANGES

* - The `hello_name`, `from_email`, `smtp_timeout`, `retries` and `proxy` settings have been moved to inside the new `verif_method` field, which is now the centralized place to configure how each email is verified (categorized by email provider).



# [0.10.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.9.1...v0.10.0) (2024-12-15)


* feat(core)!: Update async-smtp to 0.9 (#1520) ([297ce4f](https://github.com/reacherhq/check-if-email-exists/commit/297ce4f11994b483faa015bebe4abf550eb77e11)), closes [#1520](https://github.com/reacherhq/check-if-email-exists/issues/1520)
* feat!: Add `/v1/{check_email,bulk}` endpoints with throttle&concurrency (#1537) ([08522e4](https://github.com/reacherhq/check-if-email-exists/commit/08522e4326bbcbc980cf501d5d994d0c17222561)), closes [#1537](https://github.com/reacherhq/check-if-email-exists/issues/1537)
* fix(core)!: Clean up CheckEmailInput (#1531) ([b97b9ff](https://github.com/reacherhq/check-if-email-exists/commit/b97b9ff9b91bdfbf18e5c0892559e87e7cd5e16c)), closes [#1531](https://github.com/reacherhq/check-if-email-exists/issues/1531)
* refactor!: Use config-rs instead of env vars (#1530) ([bcd2dc8](https://github.com/reacherhq/check-if-email-exists/commit/bcd2dc867b7dc2bdaeb70097fd14109c2a40da17)), closes [#1530](https://github.com/reacherhq/check-if-email-exists/issues/1530)
* feat(backend)!: Remove /v0/bulk endpoints (#1421) ([522f324](https://github.com/reacherhq/check-if-email-exists/commit/522f32448416cd75a70ddb51038e50d06c3130b4)), closes [#1421](https://github.com/reacherhq/check-if-email-exists/issues/1421)
* fix!(core): Bump timeout to 45s, set retries to 1 (#1406) ([22e8e3e](https://github.com/reacherhq/check-if-email-exists/commit/22e8e3e86ce922e76262f33ceeec2388334a5264)), closes [#1406](https://github.com/reacherhq/check-if-email-exists/issues/1406)
* refactor!: Use verify method for known providers (#1366) ([5ca4dfa](https://github.com/reacherhq/check-if-email-exists/commit/5ca4dfa5ec38fba0ec7cfb052106da8d6af4df44)), closes [#1366](https://github.com/reacherhq/check-if-email-exists/issues/1366)


### Bug Fixes

* Add backend_name in /v0/check_email ([a738fae](https://github.com/reacherhq/check-if-email-exists/commit/a738faec99942d20b817298f7850e84ab3e74835))
* Add HoneyPot rule ([fb428ef](https://github.com/reacherhq/check-if-email-exists/commit/fb428ef42586641711dfd10190514ff5aa24583d))
* **backend:** CSV download retrieves all results ([#1362](https://github.com/reacherhq/check-if-email-exists/issues/1362)) ([b3670fc](https://github.com/reacherhq/check-if-email-exists/commit/b3670fcaebce05a0aab09bcc3253134cb3c643c1))
* **backend:** Fix docker CTRL+C ([3a7245f](https://github.com/reacherhq/check-if-email-exists/commit/3a7245f9a47e8332d682d437d9492559e5adf66f))
* **backend:** Fix env var for multiple queues ([ed19166](https://github.com/reacherhq/check-if-email-exists/commit/ed191662b18c62f397b4fed6b95249b5aa76c423))
* **backend:** Update sqlx to 0.7 ([#1390](https://github.com/reacherhq/check-if-email-exists/issues/1390)) ([7198f87](https://github.com/reacherhq/check-if-email-exists/commit/7198f87de92ab403cdc1e7c68667cdef9db96085))
* **ci:** actions/download-artifact@v4 ([ec48fec](https://github.com/reacherhq/check-if-email-exists/commit/ec48fec4bd675cbc33198e27e51b2d5c1f9090b5))
* **ci:** Fix Windows build ([#1397](https://github.com/reacherhq/check-if-email-exists/issues/1397)) ([ab2bb41](https://github.com/reacherhq/check-if-email-exists/commit/ab2bb4184adbd77628a38de6dccf01a1fde029cb))
* **ci:** Use v4 of upload-artifacts ([fa7f438](https://github.com/reacherhq/check-if-email-exists/commit/fa7f438d3afa7b132765107d16f41d6ce7d3b4d9))
* **ci:** Use v4 of upload-artifacts ([b97d181](https://github.com/reacherhq/check-if-email-exists/commit/b97d181b42c72f7a94dfb86acada09d423af1c0e))
* **core:** Fix gmail test ([ea80690](https://github.com/reacherhq/check-if-email-exists/commit/ea80690b4168485ed7e03f4e228a12e276d605b0))
* **core:** Fix hotmail/outlook checks ([5e4bf16](https://github.com/reacherhq/check-if-email-exists/commit/5e4bf16e75e01ba17dd9022934359c9d03f3b0c8))
* **core:** Headless check for Microsoft365 too ([#1346](https://github.com/reacherhq/check-if-email-exists/issues/1346)) ([682cc2d](https://github.com/reacherhq/check-if-email-exists/commit/682cc2d96b93d73f3fca3ba11f03800477c8fb9e))
* **core:** More robust Hotmail invalid check ([ee741f4](https://github.com/reacherhq/check-if-email-exists/commit/ee741f4570050f559395e687da64c64ff9046afb))
* **core:** Prefer empty MX lookup when Err NoRecordsFound ([#1409](https://github.com/reacherhq/check-if-email-exists/issues/1409)) ([d4b5ef9](https://github.com/reacherhq/check-if-email-exists/commit/d4b5ef9696a8c3ff0eaad2d3b5321437bd2a4df3))
* **core:** Use semver in sentry ([03e6c97](https://github.com/reacherhq/check-if-email-exists/commit/03e6c97a7f842b115b367ca942119496d8400024))
* **core:** Use Smtp for Gmail by default ([8e79884](https://github.com/reacherhq/check-if-email-exists/commit/8e79884314f0c1eec5a7964fa686e2c60e7d2209))
* **core:** Use tagged enum representation ([ffde851](https://github.com/reacherhq/check-if-email-exists/commit/ffde851068798adc3372d843a916a121b5caeccb))
* Fix dockerfile ([ce5067e](https://github.com/reacherhq/check-if-email-exists/commit/ce5067e4050e0cf3fa6c022bc7e25e5f15261c2a))
* Fix dockerfile ([83d70d8](https://github.com/reacherhq/check-if-email-exists/commit/83d70d8886730795ff69320e6ebd8e40fdf18d5e))
* Fix dockerfile build ([95aeecb](https://github.com/reacherhq/check-if-email-exists/commit/95aeecbdc9d712a5e7f9e0d547f68da4fa602d61))
* Fix Dockerfiles ([e9fb1e3](https://github.com/reacherhq/check-if-email-exists/commit/e9fb1e33435f89d627d89b98b358f257325dc13b))
* Fix duplicate `yahoo_verif_method` field in default() inputs ([#1428](https://github.com/reacherhq/check-if-email-exists/issues/1428)) ([b7c51d5](https://github.com/reacherhq/check-if-email-exists/commit/b7c51d5caaf21140c174cb419aedaf8fe752f817))
* Only do headless for non-365 hotmail emails ([1c52bdc](https://github.com/reacherhq/check-if-email-exists/commit/1c52bdc75fb201f2e54c62d5f67f50a56c57cb83))
* Put Smtp debug details in Debug struct ([5b71ca5](https://github.com/reacherhq/check-if-email-exists/commit/5b71ca59b6fab18263348aeafc7a895b7f4b8076))
* Remove local_ip retrieval ([ff8e599](https://github.com/reacherhq/check-if-email-exists/commit/ff8e5998f8b88954b4104f9251d1331542dbb182))
* Revert Cargo files ([#1389](https://github.com/reacherhq/check-if-email-exists/issues/1389)) ([96a2278](https://github.com/reacherhq/check-if-email-exists/commit/96a2278823ce717f9b1e79feccd13c059a598906))
* rm .rustfmt.toml ([#1524](https://github.com/reacherhq/check-if-email-exists/issues/1524)) ([1691d2d](https://github.com/reacherhq/check-if-email-exists/commit/1691d2db73b5dbd7384a0a99c60b4878be2aae1b))
* Support queues in env var ([39655d5](https://github.com/reacherhq/check-if-email-exists/commit/39655d51afe5f65d62cd5dc3485586e16bcdec31))
* Typo in expect of RCH_VERIF_METHOD ([#1405](https://github.com/reacherhq/check-if-email-exists/issues/1405)) ([c50d8eb](https://github.com/reacherhq/check-if-email-exists/commit/c50d8ebdfc470fe1ec6290e07668c70095298799))


### Features

* Add back RabbitMQ-based worker ([#1513](https://github.com/reacherhq/check-if-email-exists/issues/1513)) ([de75ece](https://github.com/reacherhq/check-if-email-exists/commit/de75eceef32c6ea512e0a301ec62d393bb59ff0f))
* Add debug information about each email verification ([#1391](https://github.com/reacherhq/check-if-email-exists/issues/1391)) ([3ea6e66](https://github.com/reacherhq/check-if-email-exists/commit/3ea6e6607735682dfca6ecfa27460650ac6e42d3))
* Add proxy field in SmtpDebug ([2f60a03](https://github.com/reacherhq/check-if-email-exists/commit/2f60a03f25d56397eb54302b134730ef923d9105))
* Add RabbitMQ worker ([#1395](https://github.com/reacherhq/check-if-email-exists/issues/1395)) ([ecef8c9](https://github.com/reacherhq/check-if-email-exists/commit/ecef8c98deb744390c7017a4e98d4f3c7e737fcb))
* Add sentry logging to worker ([5aa6026](https://github.com/reacherhq/check-if-email-exists/commit/5aa6026e6147fd68f9b93c4feb2752b51c337aae))
* Allow /v1/check_email without worker mode ([9ca9f39](https://github.com/reacherhq/check-if-email-exists/commit/9ca9f39ee487dc1b7d9b4cdc9a0b2c0669b10bc0))
* **backend:** Add one simple retry on Unknown ([fcffc1a](https://github.com/reacherhq/check-if-email-exists/commit/fcffc1a28bab990b0596ad8b66163e47a494191b))
* **backend:** Add POST /v1/bulk ([#1413](https://github.com/reacherhq/check-if-email-exists/issues/1413)) ([d9302d4](https://github.com/reacherhq/check-if-email-exists/commit/d9302d4c1cec6a5a1788afe2a3718df8986f118f))
* **backend:** Add reply-to queue ([aaea59f](https://github.com/reacherhq/check-if-email-exists/commit/aaea59f251634db7c35f029b09ef6e5f8c77cfbc))
* **backend:** Add worker webhook ([db90cfa](https://github.com/reacherhq/check-if-email-exists/commit/db90cfa27b85916685268a3599bdfdb2c46de07a))
* **backend:** Customize SMTP defaults ([8f152b8](https://github.com/reacherhq/check-if-email-exists/commit/8f152b83c70b94618b71308552a6999f4b27aa2f))
* **backend:** Prune bulk email verification database ([#1377](https://github.com/reacherhq/check-if-email-exists/issues/1377)) ([f905735](https://github.com/reacherhq/check-if-email-exists/commit/f90573566abf40133ebfb28ebc8f18ad8278a9b3))
* **backend:** Reject a request with to_email field empty or missing ([#1353](https://github.com/reacherhq/check-if-email-exists/issues/1353)) ([1d9c29f](https://github.com/reacherhq/check-if-email-exists/commit/1d9c29f5a48655a11f985b7df91c8bcbdf102487))
* **backend:** Support RCH_SMTP_TIMEOUT ([#1407](https://github.com/reacherhq/check-if-email-exists/issues/1407)) ([b9bda40](https://github.com/reacherhq/check-if-email-exists/commit/b9bda4049540372811a86d8dd7ba873c9875e54d))
* **core:** Add domain-specific rules as JSON file ([#1347](https://github.com/reacherhq/check-if-email-exists/issues/1347)) ([cab143c](https://github.com/reacherhq/check-if-email-exists/commit/cab143c72889c585adbf041e9c248e57d0c4c4ca))
* **core:** Bump to 45s timeout for some domains ([#1348](https://github.com/reacherhq/check-if-email-exists/issues/1348)) ([fda33a2](https://github.com/reacherhq/check-if-email-exists/commit/fda33a27441e2ccb1c4e97c0fc582abf25b1561f))
* **core:** Default Gmail checks to use API ([4304743](https://github.com/reacherhq/check-if-email-exists/commit/4304743fa93b6511857827afcdaa1fb9124bd62b))
* Increase content length limit for bulk validation endpoint ([#1525](https://github.com/reacherhq/check-if-email-exists/issues/1525)) ([bbdab31](https://github.com/reacherhq/check-if-email-exists/commit/bbdab31e0dde54d21f4eeb5880ae28e60de7dced))
* Update parser.rs ([#1345](https://github.com/reacherhq/check-if-email-exists/issues/1345)) ([8269f22](https://github.com/reacherhq/check-if-email-exists/commit/8269f22f73214412f154927a908a7769d3f8b00c))
* Use 2 queues instead of 1 ([#1396](https://github.com/reacherhq/check-if-email-exists/issues/1396)) ([af44f6c](https://github.com/reacherhq/check-if-email-exists/commit/af44f6c6629267571e7754a8c40c1036dbf4fc7d))
* Yahoo account recovery via headless ([#1364](https://github.com/reacherhq/check-if-email-exists/issues/1364)) ([6f0f12b](https://github.com/reacherhq/check-if-email-exists/commit/6f0f12b8cf528e819f8743f7e3c5f5e141c51559))


### Reverts

* **backend:** Bring back the sqlxmq-based bulk verification ([#1477](https://github.com/reacherhq/check-if-email-exists/issues/1477)) ([322ad4e](https://github.com/reacherhq/check-if-email-exists/commit/322ad4e4b53d534a8ae6461f3d3383d67b219b5d)), closes [#1421](https://github.com/reacherhq/check-if-email-exists/issues/1421)


### BREAKING CHANGES

* The `smtp_security` field has been removed from the /check_email request.
* - In `/v0/check_email` endpoint, the `hotmail_verif_method` field has been replaced two fields: `hotmailb2b_verif_method` and `hotmailb2c_verif_method`
- All serializations of `"Smtp","Api","Headless"` have been converted to lowercase `"smtp","api","headless"`
* We switched to a more commonly-used builder pattern to create an input:
```diff
- let mut input = CheckEmailInput::new("someone@gmail.com");
- input.set_from_email("me@mycompany.com");

+ let input = CheckEmailInputBuilder::default()
+     .to_email("someone@gmail.com")
+     .from_email("me@mycompany.com")
+     .build()
+     .unwrap();

let res = check_email(input, &config).await;
```
* The main function, `check_email()`, now takes a second argument called `ReacherConfig`. This struct contains configuration such as the webdriver address to listen to for headless email verifications, or an optional Sentry configuration to send error reports to. Previously, these configurations were passed through poorly-documented environment variables; now we make them explicit. When migration, you can pass `ReacherConfig::default()` which returns sensible default values.
* Remove /v0/bulk endpoints in favor of the /v1/bulk endpoints. New docs are here: https://help.reacher.email/bulk-email-verification.
* Vercel functions (used by https://app.reacher.email) usually timeout with a 504 error within less than 60s. So we should absolutely make a verification in less time than that.

After some testing, Reacher performs better with this setting:
- each SMTP connection times out after 45s, but we don't retry
over this previous setting
- each SMTP connection times out after ~20s, but we do retry once (to avoid greylisting in some rare cases)

Changing the default behaviour in this PR.
* For Hotmail, Gmail and Yahoo addresses, the `*_use_api` and `*_use_headless` parameters have been removed and replaced with a `*VerifyMethod`, an enum which can take value Api, Headless or Smtp. If using headless, pass a webdriver address to env variable RCH_WEBDRIVER_ADDR. 
* `input.hotmail_use_headless` is now a bool instead of a string. Pass the webdriver address as an environment variable `RCH_WEBDRIVER_ADDR` now.
* **core:** `SmtpError::TimeoutError` has been removed in favor of the one async-smtp uses, namely `std::io::Error` with `ErrorKind::TimeoutError`



## [0.9.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.32...v0.9.1) (2023-10-08)


* refactor!: Change RUST_LOG target to `reacher` (#1152) ([7e87be2](https://github.com/reacherhq/check-if-email-exists/commit/7e87be26f1e35a6936bfc967c872cd42b93fd256)), closes [#1152](https://github.com/reacherhq/check-if-email-exists/issues/1152)


### Bug Fixes

* **backend:** Fix CI priting ([748940c](https://github.com/reacherhq/check-if-email-exists/commit/748940ca2fa7fb59aac8e07a408a22d1ab688527))
* **backend:** Fix deploy to docker ([20fcfa6](https://github.com/reacherhq/check-if-email-exists/commit/20fcfa6032e4614dc459a34183958fde63199acf))
* **backend:** Fix dockerfile ([f0ed49f](https://github.com/reacherhq/check-if-email-exists/commit/f0ed49f50238c1c71a130f3db19ec047af00b8df))
* **backend:** Improve sentry error messages ([#1155](https://github.com/reacherhq/check-if-email-exists/issues/1155)) ([d90d998](https://github.com/reacherhq/check-if-email-exists/commit/d90d998d1cb189fed3f888659aa08fd4fabf6e93))
* **backend:** Redact email in sentry bug tracking ([2c2d1d8](https://github.com/reacherhq/check-if-email-exists/commit/2c2d1d88c0086196bc09359e32c96638124d9539))
* **cli:** Update flags default values ([a4fe57e](https://github.com/reacherhq/check-if-email-exists/commit/a4fe57e9ab89659e12182719ccb12fb2cdcb5f2e))
* **core:** Add more invalid parsing and improve logging ([#1156](https://github.com/reacherhq/check-if-email-exists/issues/1156)) ([b5ae9f8](https://github.com/reacherhq/check-if-email-exists/commit/b5ae9f8ad910b77ad6a179ecb5d4b633011ed2f4))
* **core:** Default SMTP timeout to 15 ([0d4fa4d](https://github.com/reacherhq/check-if-email-exists/commit/0d4fa4d8f662ecfd3fa2e0359322f324a8ef86db))
* **core:** Don't use headless on Microsoft 465 addresses ([#1196](https://github.com/reacherhq/check-if-email-exists/issues/1196)) ([0c3c21d](https://github.com/reacherhq/check-if-email-exists/commit/0c3c21daf6ea79875835121fb86ab7c0c86d55eb))
* **core:** Fix default CheckEmailInput ([09215a1](https://github.com/reacherhq/check-if-email-exists/commit/09215a13ac3525861e6cd1dea3fc71c13dfffe52))
* **core:** Fix hotmail headless option parsing ([6ddc3b9](https://github.com/reacherhq/check-if-email-exists/commit/6ddc3b96da0d01b02711d62873ad0d0df6bf1b33))
* **core:** Fix hotmail headless with authenticator ([51cdb2e](https://github.com/reacherhq/check-if-email-exists/commit/51cdb2e3c13a433fff92f1d3dcf1bfcb90f6ce7b))
* **core:** Fix MX random record selection ([#1263](https://github.com/reacherhq/check-if-email-exists/issues/1263)) ([9fae593](https://github.com/reacherhq/check-if-email-exists/commit/9fae593b8590ad5efb3e7d16bbd25cc05c228cb9))
* **core:** Improve invalid parser ([#1166](https://github.com/reacherhq/check-if-email-exists/issues/1166)) ([bb46004](https://github.com/reacherhq/check-if-email-exists/commit/bb460046bf1cb031fee706d836c8a737157f803c))
* **core:** Improve parser and headless hotmail runner ([#1167](https://github.com/reacherhq/check-if-email-exists/issues/1167)) ([0de33a5](https://github.com/reacherhq/check-if-email-exists/commit/0de33a5f265105a769c7ca6125df0fd4f88b89e2))
* **core:** Improve parser from Sentry errors ([fbaf588](https://github.com/reacherhq/check-if-email-exists/commit/fbaf58824a339e546d50c2125a459161769dda6e))
* **core:** Improve parser's `is_invalid` ([#1159](https://github.com/reacherhq/check-if-email-exists/issues/1159)) ([ec1c4d5](https://github.com/reacherhq/check-if-email-exists/commit/ec1c4d5e5d4c94d75d255a0699402f75eb29f7ab))
* **core:** No sandbox in headless Hotmail check ([0590438](https://github.com/reacherhq/check-if-email-exists/commit/0590438310f3c052b2748a8c408e0d8dbfb777b7))
* **core:** Remove antispam check ([#1337](https://github.com/reacherhq/check-if-email-exists/issues/1337)) ([06f18ed](https://github.com/reacherhq/check-if-email-exists/commit/06f18edf7aee5640b3725feedfa7b7f213da83a8))
* **core:** Yahoo add back IDENTIFIER_EXISTS ([2b63556](https://github.com/reacherhq/check-if-email-exists/commit/2b635564efb37b0aa891bbba77244e6cf2d611bb))
* **core:** yahoo api changes: yid is userId now, sessionIndex is required and fo… ([#1314](https://github.com/reacherhq/check-if-email-exists/issues/1314)) ([0209111](https://github.com/reacherhq/check-if-email-exists/commit/02091115026520596fc5b4b2a6757169e91cba15))
* Don't auto-fetch Chrome, install in Docker ([84fcc0d](https://github.com/reacherhq/check-if-email-exists/commit/84fcc0de40567126ce3a385934086450c3a89ccf))
* split Microsoft 365/Hotmail functionality ([#1204](https://github.com/reacherhq/check-if-email-exists/issues/1204)) ([e987b13](https://github.com/reacherhq/check-if-email-exists/commit/e987b13a5ccd98d28fb756f1bf41427c337750c4))
* Switch back to upstream fast-socks ([#1164](https://github.com/reacherhq/check-if-email-exists/issues/1164)) ([db356f1](https://github.com/reacherhq/check-if-email-exists/commit/db356f19374843ca135de8ebd8a6c34bfeb017a8))
* TLS accept unsafe ([778692b](https://github.com/reacherhq/check-if-email-exists/commit/778692bce760c0a1e1201dd3e11b41e7ccb7e2e8))
* Use chromedriver instead of gecko for parallel requests ([e282e28](https://github.com/reacherhq/check-if-email-exists/commit/e282e28aeb7259d800f7faad97173c3a216095a4))


### Features

* **#289:** add haveibeenpwned check ([#1253](https://github.com/reacherhq/check-if-email-exists/issues/1253)) ([166dbd2](https://github.com/reacherhq/check-if-email-exists/commit/166dbd2cc878e30c51538b919abc1aaea4465c45)), closes [#289](https://github.com/reacherhq/check-if-email-exists/issues/289)
* add email address normalisation ([#1206](https://github.com/reacherhq/check-if-email-exists/issues/1206)) ([f8ec348](https://github.com/reacherhq/check-if-email-exists/commit/f8ec348883cd4f4a20a8acbb38d54b69e798222b)), closes [#952](https://github.com/reacherhq/check-if-email-exists/issues/952)
* add Microsoft 365 HTTP API validation ([#1194](https://github.com/reacherhq/check-if-email-exists/issues/1194)) ([5d3c49f](https://github.com/reacherhq/check-if-email-exists/commit/5d3c49f41ef1369efe2a9e63b24543e281ae0776)), closes [#937](https://github.com/reacherhq/check-if-email-exists/issues/937)
* Add skipped domains ([#1293](https://github.com/reacherhq/check-if-email-exists/issues/1293)) ([29119fa](https://github.com/reacherhq/check-if-email-exists/commit/29119fa72027c9830396bbdf3e90f08c0c89d7a7))
* Add suggestions for syntax errors ([#1192](https://github.com/reacherhq/check-if-email-exists/issues/1192)) ([2d385f3](https://github.com/reacherhq/check-if-email-exists/commit/2d385f30f7a62ab2706599fbb89fb50275cffb5f))
* additional Gmail validation ([#1193](https://github.com/reacherhq/check-if-email-exists/issues/1193)) ([49c8f5c](https://github.com/reacherhq/check-if-email-exists/commit/49c8f5c3b4a3db04533d06d7267b0f15ebda3285)), closes [#937](https://github.com/reacherhq/check-if-email-exists/issues/937)
* **backend:** Add header secret to protect against public requests ([#1158](https://github.com/reacherhq/check-if-email-exists/issues/1158)) ([fa6a56b](https://github.com/reacherhq/check-if-email-exists/commit/fa6a56b62f4b3aeeec704cfe4882755998d40833))
* **core:** Add check for antispam MX records ([#1257](https://github.com/reacherhq/check-if-email-exists/issues/1257)) ([c9771da](https://github.com/reacherhq/check-if-email-exists/commit/c9771da66c7869a4d0a255e2e2536f2863e8958c))
* **core:** Add check gravatar image ([#1188](https://github.com/reacherhq/check-if-email-exists/issues/1188)) ([6a26035](https://github.com/reacherhq/check-if-email-exists/commit/6a26035327ab681a65a4f4ba284e155f00680e89))
* **core:** Add Hotmail checks via headless password recovery ([#1165](https://github.com/reacherhq/check-if-email-exists/issues/1165)) ([7517ed9](https://github.com/reacherhq/check-if-email-exists/commit/7517ed98ba966158deebba6a1a4745c931bfed18))
* **core:** Fix disabled accts on hanmail.net ([#1339](https://github.com/reacherhq/check-if-email-exists/issues/1339)) ([90393c8](https://github.com/reacherhq/check-if-email-exists/commit/90393c8dda39267da7eb5efe6f112c8f25a593f4))
* **core:** Skip catch-all for known domains ([#1336](https://github.com/reacherhq/check-if-email-exists/issues/1336)) ([c40a46c](https://github.com/reacherhq/check-if-email-exists/commit/c40a46c4555129346bd9efa444a483bf25b679fe))
* **core:** Update default MAIL-FROM and HELO ([743a811](https://github.com/reacherhq/check-if-email-exists/commit/743a8111b4831ee19e7ac887c39a8da2775acd4c))
* Move `backend` code to this repo ([#1138](https://github.com/reacherhq/check-if-email-exists/issues/1138)) ([0dc6053](https://github.com/reacherhq/check-if-email-exists/commit/0dc60531d26efb217137347ef2b6aaf678d94238))
* Revert back to `check_email` input with single email ([#1150](https://github.com/reacherhq/check-if-email-exists/issues/1150)) ([ce1ba53](https://github.com/reacherhq/check-if-email-exists/commit/ce1ba5346849b578a0ed30b1d72096f15cfbc09d))
* Set default timeout to 10s ([#1251](https://github.com/reacherhq/check-if-email-exists/issues/1251)) ([d04f84c](https://github.com/reacherhq/check-if-email-exists/commit/d04f84cc1e7b30e02d3717ab1af9f680cdb2c27f))


### BREAKING CHANGES

* The `RUST_LOG` target has been changed from `check-if-email-exists` to `reacher`.

```diff
- RUST_LOG=check-if-email-exists=debug cargo run
- RUST_LOG=reacher=debug cargo run
```
* The library's main function `check_email`'s argument `CheckEmailInput` nows takes a single `to_email` field, instead of a `to_emails: Vec<String>`

```diff
pub struct CheckEmailInput {
- pub to_emails: Vec<String>,
+ pub to_email: String,
  // --snip--
}
```

This effectively makes the public API more similar to the v0.7.* series. I'm still thinking about how to best verify multiple emails in one SMTP connection, but it most likely will be a new function with a different API. Follow [issue #65](https://github.com/reacherhq/check-if-email-exists/issues/65) for more info.



## [0.8.32](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.31...v0.8.32) (2022-08-13)


### Bug Fixes

* Fix parsing some invalid emails ([cb65c0f](https://github.com/reacherhq/check-if-email-exists/commit/cb65c0f4767b2f163f48054652f7652b6d0b6043))
* Syntax also check using using `mailchecker` ([8385bec](https://github.com/reacherhq/check-if-email-exists/commit/8385bec6fedc0912881800442bffda5b33c2f394))


### Features

* Use opportunistic STARTTLS by default ([#1079](https://github.com/reacherhq/check-if-email-exists/issues/1079)) ([54911f0](https://github.com/reacherhq/check-if-email-exists/commit/54911f0a8ec51e753f757878021e933609cff868))



## [0.8.31](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.30...v0.8.31) (2022-08-10)



## [0.8.30](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.29...v0.8.30) (2022-06-02)


### Bug Fixes

* Fix `has_full_inbox` check too lenient ([93de444](https://github.com/reacherhq/check-if-email-exists/commit/93de444dfa7c6d66061570115be8f53f0647c431))


### Features

* Add `smtp.error.description` field for human-readable description of error ([#1111](https://github.com/reacherhq/check-if-email-exists/issues/1111)) ([43b47ea](https://github.com/reacherhq/check-if-email-exists/commit/43b47ea2b9250f2c6d58c8a0ec4340066169c169))



## [0.8.29](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.28...v0.8.29) (2022-03-02)


### Features

* Loop through all MX servers ([#1070](https://github.com/reacherhq/check-if-email-exists/issues/1070)) ([11e6a06](https://github.com/reacherhq/check-if-email-exists/commit/11e6a06a67f5893b729c76d1a33667f83d63c836))



## [0.8.28](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.27...v0.8.28) (2022-02-11)


### Features

* Add proxy username/password ([#1057](https://github.com/reacherhq/check-if-email-exists/issues/1057)) ([d9583c6](https://github.com/reacherhq/check-if-email-exists/commit/d9583c6ae0d3353a5135dd157999cf579b308d6d))



## [0.8.27](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.26...v0.8.27) (2022-02-07)


### Features

* Allow user to define SMTP client security for TLS ([#1043](https://github.com/reacherhq/check-if-email-exists/issues/1043)) ([bc722ff](https://github.com/reacherhq/check-if-email-exists/commit/bc722ff1a9b30747308a3b3b5959d73e5e853292))
* Break SmtpError into `{Helo,Connect,ConnectWithStream,MailFrom,RcptTo,Close}Error` ([#1055](https://github.com/reacherhq/check-if-email-exists/issues/1055)) ([64e5193](https://github.com/reacherhq/check-if-email-exists/commit/64e5193c48a6bf4c080e79daeefd1c98dadffd5d))



## [0.8.26](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.25...v0.8.26) (2022-01-26)


### Bug Fixes

* Use std::default for deriving ([#1015](https://github.com/reacherhq/check-if-email-exists/issues/1015)) ([03720f0](https://github.com/reacherhq/check-if-email-exists/commit/03720f027fd68d5ea5ae538aa567a621f4a65fe3))


### Features

* Add SMTP retries to avoid greylisting ([#1041](https://github.com/reacherhq/check-if-email-exists/issues/1041)) ([b451a1e](https://github.com/reacherhq/check-if-email-exists/commit/b451a1e93a6ccf025c78d56dee7439ad607c8507))



## [0.8.25](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.24...v0.8.25) (2021-10-05)


### Bug Fixes

* Use async_std_resolver::resolver_from_system_conf ([#982](https://github.com/reacherhq/check-if-email-exists/issues/982)) ([376c3b0](https://github.com/reacherhq/check-if-email-exists/commit/376c3b0d4743ccc60a1df2a9fa3e9f2f5cd68178))
* Use TLS when available ([#964](https://github.com/reacherhq/check-if-email-exists/issues/964)) ([aed11d2](https://github.com/reacherhq/check-if-email-exists/commit/aed11d2e15b6b7688ecaf856824ca6effbb5d21b))


### Features

* Add possibility to set SMTP port ([#985](https://github.com/reacherhq/check-if-email-exists/issues/985)) ([cdabdf8](https://github.com/reacherhq/check-if-email-exists/commit/cdabdf80e858908d6c33e1273dfdc1fef0f78d35))



## [0.8.24](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.23...v0.8.24) (2021-07-03)


### Features

* Add `CheckEmailInput` setter `set_` prefix to differentiate with accessing fields ([#933](https://github.com/reacherhq/check-if-email-exists/issues/933)) ([276f656](https://github.com/reacherhq/check-if-email-exists/commit/276f6561e7a98af6415dbd4645d84cbe697b738e))
* Add deprecated warning when running HTTP server ([#943](https://github.com/reacherhq/check-if-email-exists/issues/943)) ([e4b1570](https://github.com/reacherhq/check-if-email-exists/commit/e4b1570a8be5573f7394a3139f34ab021452cc3a))



## [0.8.23](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.22...v0.8.23) (2021-06-20)


### Bug Fixes

* Add serde (De)Serialize to pub structs ([#931](https://github.com/reacherhq/check-if-email-exists/issues/931)) ([949475d](https://github.com/reacherhq/check-if-email-exists/commit/949475dee4a1ed96e873688e7432c702eb30af62))



## [0.8.22](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.21...v0.8.22) (2021-03-31)



## [0.8.21](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.20...v0.8.21) (2021-03-31)



## [0.8.20](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.19...v0.8.20) (2021-03-30)



## [0.8.19](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.18...v0.8.19) (2021-01-10)


### Bug Fixes

* Reconnect auto-closed SMTP connections by foreign server ([#825](https://github.com/reacherhq/check-if-email-exists/issues/825)) ([01ccf0d](https://github.com/reacherhq/check-if-email-exists/commit/01ccf0d2363475d486bb9827e3e3b9d6954bc032))


### Features

* Consider CLI config parameters in HTTP request checks ([#827](https://github.com/reacherhq/check-if-email-exists/issues/827)) ([88b751a](https://github.com/reacherhq/check-if-email-exists/commit/88b751a17f4367c990e8a54661e3872898afd10f))



## [0.8.18](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.17...v0.8.18) (2021-01-07)


### Bug Fixes

* Check deliverability using successful response code instead of message parsing ([#822](https://github.com/reacherhq/check-if-email-exists/issues/822)) ([39d0ecd](https://github.com/reacherhq/check-if-email-exists/commit/39d0ecdeaf078dce5cdb59cba95ab9e02bce11ee))



## [0.8.17](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.16...v0.8.17) (2021-01-05)


### Bug Fixes

* Add better checks for existing mailboxes ([#819](https://github.com/reacherhq/check-if-email-exists/issues/819)) ([9f88d01](https://github.com/reacherhq/check-if-email-exists/commit/9f88d01fad2c8de898aa35645bab95a14a147393))



## [0.8.16](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.15...v0.8.16) (2020-12-07)


### Features

* Add proxy_host and proxy_port info to HTTP ([#770](https://github.com/reacherhq/check-if-email-exists/issues/770)) ([123f431](https://github.com/reacherhq/check-if-email-exists/commit/123f431e10e90339e030783582d6e4c4919c1a33))



## [0.8.15](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.14...v0.8.15) (2020-11-11)


### Bug Fixes

* Don't check inputted email if catch-all ([#714](https://github.com/reacherhq/check-if-email-exists/issues/714)) ([5129dd1](https://github.com/reacherhq/check-if-email-exists/commit/5129dd1374d3ef93db632f6d7e140e3ce69369b2))
* Fix 'reached the type-length limit while instantiating' ([#665](https://github.com/reacherhq/check-if-email-exists/issues/665)) ([fa040fd](https://github.com/reacherhq/check-if-email-exists/commit/fa040fda8b16ca4d540829ee72f9d7b07ef77fdd))



## [0.8.14](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.13...v0.8.14) (2020-09-24)


### Bug Fixes

* Add more known errors for invalid email ([#543](https://github.com/reacherhq/check-if-email-exists/issues/543)) ([ad209c7](https://github.com/reacherhq/check-if-email-exists/commit/ad209c7ecb3f5aa466f31e293a05734b5edf5f6a))


### Features

* Add optional timeout on smtp verification ([#611](https://github.com/reacherhq/check-if-email-exists/issues/611)) ([c70de7d](https://github.com/reacherhq/check-if-email-exists/commit/c70de7dcac1811596c78e14888a4258e9db408ed))



## [0.8.13](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.12...v0.8.13) (2020-08-04)


### Bug Fixes

* **ci:** Put lto flag in cargo.toml ([#531](https://github.com/reacherhq/check-if-email-exists/issues/531)) ([00cbc1f](https://github.com/reacherhq/check-if-email-exists/commit/00cbc1fd46743c7579809a09b3897259213af496))



## [0.8.12](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.11...v0.8.12) (2020-08-04)


### Bug Fixes

* Add "recipient address accepted" check ([#489](https://github.com/reacherhq/check-if-email-exists/issues/489)) ([5d1e72a](https://github.com/reacherhq/check-if-email-exists/commit/5d1e72ae165f335ab97a96c3806e3293289187a2))
* http request body to use `to_emails` ([#502](https://github.com/reacherhq/check-if-email-exists/issues/502)) ([36aed56](https://github.com/reacherhq/check-if-email-exists/commit/36aed567cf705ef8d20489b2275e3d8ba58b75bb))



## [0.8.11](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.10...v0.8.11) (2020-07-11)


### Bug Fixes

* Add "Invalid email address" check ([#471](https://github.com/reacherhq/check-if-email-exists/issues/471)) ([3b03617](https://github.com/reacherhq/check-if-email-exists/commit/3b03617b81a1f9f6bc1bc6edc8c5d6d9f87eabbb))
* Add possibility to use proxy in Yahoo API request ([#472](https://github.com/reacherhq/check-if-email-exists/issues/472)) ([aafcedf](https://github.com/reacherhq/check-if-email-exists/commit/aafcedf9b9135a6550e7aa2da5d7ca5898da9b53))



## [0.8.10](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.9...v0.8.10) (2020-07-04)


### Bug Fixes

* Correct message parsing for "receiving at a rate" error ([#462](https://github.com/reacherhq/check-if-email-exists/issues/462)) ([4b31706](https://github.com/reacherhq/check-if-email-exists/commit/4b31706228a6e81852505ec21a0f70d5472b1385))



## [0.8.9](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.8...v0.8.9) (2020-07-04)


### Features

* Make using Yahoo API optional ([#460](https://github.com/reacherhq/check-if-email-exists/issues/460)) ([1e42f0a](https://github.com/reacherhq/check-if-email-exists/commit/1e42f0abef27dcea9a467f677ef9a080a3cc0f18))



## [0.8.8](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.7...v0.8.8) (2020-06-28)


### Bug Fixes

* Add debug logs for Yahoo ([e534670](https://github.com/reacherhq/check-if-email-exists/commit/e53467006f9fa435993ea58b1753ce5baa059d2a))



## [0.8.7](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.6...v0.8.7) (2020-06-28)


### Bug Fixes

* Add "recipient unknown" check ([#446](https://github.com/reacherhq/check-if-email-exists/issues/446)) ([deca071](https://github.com/reacherhq/check-if-email-exists/commit/deca071583e34bb9c5836d5238dd51975f827cdc))



## [0.8.6](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.5...v0.8.6) (2020-06-28)


### Bug Fixes

* Add additional error check for undeliverable ([#374](https://github.com/reacherhq/check-if-email-exists/issues/374)) ([e52a8f0](https://github.com/reacherhq/check-if-email-exists/commit/e52a8f0941fd53c9b087e6e59a7018d11af72dff))
* Use HTTP requests to verify Yahoo emails ([#412](https://github.com/reacherhq/check-if-email-exists/issues/412)) ([5fad57d](https://github.com/reacherhq/check-if-email-exists/commit/5fad57d88ef92d65c7d493cdcb45eff347d6a286))



## [0.8.5](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.4...v0.8.5) (2020-05-21)


### Features

* Expose misc, syntax, mx, smtp modules ([#373](https://github.com/reacherhq/check-if-email-exists/issues/373)) ([7c1d741](https://github.com/reacherhq/check-if-email-exists/commit/7c1d741f00b3a807b190140a1d91a42bce29470c))



## [0.8.4](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.3...v0.8.4) (2020-05-19)


### Bug Fixes

* Fix is_reachable with wrong syntax ([#352](https://github.com/reacherhq/check-if-email-exists/issues/352)) ([b0f0209](https://github.com/reacherhq/check-if-email-exists/commit/b0f02090edc0bb8947ab826415fa3bf8b5db55f0))



## [0.8.3](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.2...v0.8.3) (2020-05-12)


### Bug Fixes

* Lowercase Reachable enum variants ([#351](https://github.com/reacherhq/check-if-email-exists/issues/351)) ([b88c20e](https://github.com/reacherhq/check-if-email-exists/commit/b88c20ef5bc947ecd8cc646a9e6c583df0bef4d7))



## [0.8.2](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.1...v0.8.2) (2020-05-12)


### Bug Fixes

* Add "Unknown user" smtp error check ([#347](https://github.com/reacherhq/check-if-email-exists/issues/347)) ([47eb578](https://github.com/reacherhq/check-if-email-exists/commit/47eb5780f692f54aadf264b107996bb2d1a31a56))
* Add more error strings matching ([#323](https://github.com/reacherhq/check-if-email-exists/issues/323)) ([f5392d4](https://github.com/reacherhq/check-if-email-exists/commit/f5392d4befcee6e4d935e1585066eae3b57ec6fa))


### Features

* Add `is_reachable` top field ([#350](https://github.com/reacherhq/check-if-email-exists/issues/350)) ([e7abb17](https://github.com/reacherhq/check-if-email-exists/commit/e7abb17ef29610fbe9210f42830c0ba02bb130b7))
* Detect role-based accounts ([#348](https://github.com/reacherhq/check-if-email-exists/issues/348)) ([7c612fd](https://github.com/reacherhq/check-if-email-exists/commit/7c612fda110729ece094d0b022db05fa4e6b27b4))



## [0.8.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.0...v0.8.1) (2020-05-09)


### Bug Fixes

* Lowercase the error string before matching ([#321](https://github.com/reacherhq/check-if-email-exists/issues/321)) ([d983b2f](https://github.com/reacherhq/check-if-email-exists/commit/d983b2fe960ed46c4bd03c55b39d7ea58be5124f))



# [0.8.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.7.1...v0.8.0) (2020-05-08)


* refactor!: Rename main function to `check_email` (#319) ([bd12b7d](https://github.com/reacherhq/check-if-email-exists/commit/bd12b7dbbd6c090fcdf80e3d6bbe475cd1d82b9a)), closes [#319](https://github.com/reacherhq/check-if-email-exists/issues/319)


### Bug Fixes

* Rename valid_format to is_valid_syntax ([#288](https://github.com/reacherhq/check-if-email-exists/issues/288)) ([eae6482](https://github.com/reacherhq/check-if-email-exists/commit/eae64821c31d0193f77d9137ec4e7d6131f91ccb))


### BREAKING CHANGES

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



## [0.7.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.7.0...v0.7.1) (2020-04-14)


### Features

* Add possibility to verify emails via proxy ([#286](https://github.com/reacherhq/check-if-email-exists/issues/286)) ([a0ab93f](https://github.com/reacherhq/check-if-email-exists/commit/a0ab93fde5105d594a8280b942d337ff76fbb517))



# [0.7.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.6.7...v0.7.0) (2020-03-26)


### Features

* Use builder pattern for EmailInput ([#254](https://github.com/reacherhq/check-if-email-exists/issues/254)) ([0c85d36](https://github.com/reacherhq/check-if-email-exists/commit/0c85d36cdccb37d8da9566f7e7baf5dbbd266740))


### BREAKING CHANGES

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



## [0.6.7](https://github.com/reacherhq/check-if-email-exists/compare/v0.6.6...v0.6.7) (2020-03-20)



## [0.6.6](https://github.com/reacherhq/check-if-email-exists/compare/v0.6.1...v0.6.6) (2020-03-01)


### Bug Fixes

* Allow http to listen to $PORT env variable ([#215](https://github.com/reacherhq/check-if-email-exists/issues/215)) ([3b0c262](https://github.com/reacherhq/check-if-email-exists/commit/3b0c262763bc9d52855ced90aa2a435a97d35d8b))



## [0.6.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.6.0...v0.6.1) (2020-02-18)


### Features

* Add --http-host flag to CLI ([#197](https://github.com/reacherhq/check-if-email-exists/issues/197)) ([55657b2](https://github.com/reacherhq/check-if-email-exists/commit/55657b251fcc22fad2ae53da4f62a017ff01d035))



# [0.6.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.5.0...v0.6.0) (2019-12-01)


### Features

* Add a HTTP server behind the `--http` flag ([#85](https://github.com/reacherhq/check-if-email-exists/issues/85)) ([d8b733e](https://github.com/reacherhq/check-if-email-exists/commit/d8b733e5a571c512644b34219b5f2dfd9dc717b3))
* Add Dockerfile & `x86_64-unknown-linux-musl` target ([#86](https://github.com/reacherhq/check-if-email-exists/issues/86)) ([cba1241](https://github.com/reacherhq/check-if-email-exists/commit/cba124110be04d7febfeab68a6b825197b3aa1fb))


### BREAKING CHANGES

* - The `is_disposable` subfield has been moved from the `mx` field to a separate `misc` field



# [0.5.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.4.0...v0.5.0) (2019-11-16)


### Code Refactoring

* Use futures ([#78](https://github.com/reacherhq/check-if-email-exists/issues/78)) ([0e1f6b0](https://github.com/reacherhq/check-if-email-exists/commit/0e1f6b014929bbdd97eeb687e8399e016168c304))


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



# [0.4.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.3.2...v0.4.0) (2019-09-30)


### Features

* Add disposable email check ([#64](https://github.com/reacherhq/check-if-email-exists/issues/64)) ([1b2cea3](https://github.com/reacherhq/check-if-email-exists/commit/1b2cea3a6ffec08e63c5b6e7d9b2cce9d3b3c427))


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



## [0.3.2](https://github.com/reacherhq/check-if-email-exists/compare/v0.3.1...v0.3.2) (2019-09-26)


### Bug Fixes

* **core:** SyntaxError also is type & message ([#60](https://github.com/reacherhq/check-if-email-exists/issues/60)) ([996633b](https://github.com/reacherhq/check-if-email-exists/commit/996633b1ccde5dd79ec42001b6d445aa195002ad))



## [0.3.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.3.0...v0.3.1) (2019-09-26)


### Bug Fixes

* Don't use virtual workspace, be able to build ([#59](https://github.com/reacherhq/check-if-email-exists/issues/59)) ([6c93893](https://github.com/reacherhq/check-if-email-exists/commit/6c93893273483ab027af3ef769ab1246dfab7ad7))



# [0.3.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.2.3...v0.3.0) (2019-09-26)


### Features

* New error JSON format ([#56](https://github.com/reacherhq/check-if-email-exists/issues/56)) ([fec4315](https://github.com/reacherhq/check-if-email-exists/commit/fec43156b3edef30d449c14572043e312335c01b))
* Output JSON information with CLI ([#53](https://github.com/reacherhq/check-if-email-exists/issues/53)) ([1d026d5](https://github.com/reacherhq/check-if-email-exists/commit/1d026d5d5df3c1684acb30379a3640528b572485))
* Return Result<EmailDetails> instead of Result<bool>, with much more details ([#23](https://github.com/reacherhq/check-if-email-exists/issues/23)) ([39b13f5](https://github.com/reacherhq/check-if-email-exists/commit/39b13f55249cdf68e627d23cc4eee1146186d55c))



## [0.2.3](https://github.com/reacherhq/check-if-email-exists/compare/v0.2.2...v0.2.3) (2019-05-09)


### Bug Fixes

* Update version to correct version in cli ([992777c](https://github.com/reacherhq/check-if-email-exists/commit/992777ce898013f2ff998f0dc72c0308eac9d318))



## [0.2.2](https://github.com/reacherhq/check-if-email-exists/compare/v0.2.1...v0.2.2) (2019-05-09)


### Bug Fixes

* Fix travis and appveyor to build binaries ([f743e67](https://github.com/reacherhq/check-if-email-exists/commit/f743e6767a401a9d97f5f98d56a3dbbb7a38a289))



## [0.2.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.2.0...v0.2.1) (2019-05-09)


### Bug Fixes

* Refactor app to make travis build binaries ([#17](https://github.com/reacherhq/check-if-email-exists/issues/17)) ([9616ef5](https://github.com/reacherhq/check-if-email-exists/commit/9616ef5bcf6c016dd065550739b9eece5a6b8a07))



# [0.2.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.1.1...v0.2.0) (2019-05-09)


### Features

* Add serverless function ([#15](https://github.com/reacherhq/check-if-email-exists/issues/15)) ([532c4eb](https://github.com/reacherhq/check-if-email-exists/commit/532c4ebcb4a9ee13c1d3ab557085971ee774a158))
* Return Option<bool> instead of bool ([#13](https://github.com/reacherhq/check-if-email-exists/issues/13)) ([2aef345](https://github.com/reacherhq/check-if-email-exists/commit/2aef3458f694bd8deeedb6860278846650096f50))



## [0.1.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.1.0...v0.1.1) (2018-12-29)



