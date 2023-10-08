# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.9.1](https://github.com/reacherhq/check-if-email-exists/compare/v0.9.0...v0.9.1) (2023-10-08)


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
* Set default timeout to 10s ([#1251](https://github.com/reacherhq/check-if-email-exists/issues/1251)) ([d04f84c](https://github.com/reacherhq/check-if-email-exists/commit/d04f84cc1e7b30e02d3717ab1af9f680cdb2c27f))


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

## [0.9.0](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.32...v0.9.0) (2022-08-15)


### ⚠ BREAKING CHANGES

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
* The `--http` flag has been removed from `check-if-email-exists` CLI. To run a HTTP backend, please refer to the [backend](./backend) subfolder.

### Features

* Move `backend` code to this repo ([#1138](https://github.com/reacherhq/check-if-email-exists/issues/1138)) ([0dc6053](https://github.com/reacherhq/check-if-email-exists/commit/0dc60531d26efb217137347ef2b6aaf678d94238))
* Revert back to `check_email` input with single email ([#1150](https://github.com/reacherhq/check-if-email-exists/issues/1150)) ([ce1ba53](https://github.com/reacherhq/check-if-email-exists/commit/ce1ba5346849b578a0ed30b1d72096f15cfbc09d)), closes [#65](https://github.com/reacherhq/check-if-email-exists/issues/65)


* Change RUST_LOG target to `reacher` ([#1152](https://github.com/reacherhq/check-if-email-exists/issues/1152)) ([7e87be2](https://github.com/reacherhq/check-if-email-exists/commit/7e87be26f1e35a6936bfc967c872cd42b93fd256))
* Remove HTTP backend from CLI ([#1151](https://github.com/reacherhq/check-if-email-exists/issues/1151)) ([7184372](https://github.com/reacherhq/check-if-email-exists/commit/71843720c9b87fa0e43fa482a35ef074435bf562))

### [0.8.32](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.31...v0.8.32) (2022-08-13)


### Features

* Use opportunistic STARTTLS by default ([#1079](https://github.com/reacherhq/check-if-email-exists/issues/1079)) ([54911f0](https://github.com/reacherhq/check-if-email-exists/commit/54911f0a8ec51e753f757878021e933609cff868))


### Bug Fixes

* Fix parsing some invalid emails ([cb65c0f](https://github.com/reacherhq/check-if-email-exists/commit/cb65c0f4767b2f163f48054652f7652b6d0b6043))
* Syntax also check using using `mailchecker` ([8385bec](https://github.com/reacherhq/check-if-email-exists/commit/8385bec6fedc0912881800442bffda5b33c2f394))

### [0.8.31](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.30...v0.8.31) (2022-08-10)

### Improvements

* Bump packages incl. `async-smtp` ([#1136](https://github.com/amaurymartiny/check-if-email-exists/issues/1136)) ([43b47ea](https://github.com/amaurymartiny/check-if-email-exists/commit/cbdab7c35bc4ce7f7da79fa9316727cc840da4e4))

### [0.8.30](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.29...v0.8.30) (2022-06-02)


### Features

* Add `smtp.error.description` field for human-readable description of error ([#1111](https://github.com/amaurymartiny/check-if-email-exists/issues/1111)) ([43b47ea](https://github.com/amaurymartiny/check-if-email-exists/commit/43b47ea2b9250f2c6d58c8a0ec4340066169c169))


### Bug Fixes

* Fix `has_full_inbox` check too lenient ([93de444](https://github.com/amaurymartiny/check-if-email-exists/commit/93de444dfa7c6d66061570115be8f53f0647c431))

### [0.8.29](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.28...v0.8.29) (2022-03-02)


### Features

* Loop through all MX servers ([#1070](https://github.com/amaurymartiny/check-if-email-exists/issues/1070)) ([11e6a06](https://github.com/amaurymartiny/check-if-email-exists/commit/11e6a06a67f5893b729c76d1a33667f83d63c836))

### [0.8.28](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.27...v0.8.28) (2022-02-11)


### Features

* Add proxy username/password ([#1057](https://github.com/amaurymartiny/check-if-email-exists/issues/1057)) ([d9583c6](https://github.com/amaurymartiny/check-if-email-exists/commit/d9583c6ae0d3353a5135dd157999cf579b308d6d))

### [0.8.27](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.26...v0.8.27) (2022-02-07)


### Features

* Allow user to define SMTP client security for TLS ([#1043](https://github.com/amaurymartiny/check-if-email-exists/issues/1043)) ([bc722ff](https://github.com/amaurymartiny/check-if-email-exists/commit/bc722ff1a9b30747308a3b3b5959d73e5e853292))
* Break SmtpError into `{Helo,Connect,ConnectWithStream,MailFrom,RcptTo,Close}Error` ([#1055](https://github.com/amaurymartiny/check-if-email-exists/issues/1055)) ([64e5193](https://github.com/amaurymartiny/check-if-email-exists/commit/64e5193c48a6bf4c080e79daeefd1c98dadffd5d))

### [0.8.26](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.25...v0.8.26) (2022-01-26)


### Features

* Add SMTP retries to avoid greylisting ([#1041](https://github.com/amaurymartiny/check-if-email-exists/issues/1041)) ([b451a1e](https://github.com/amaurymartiny/check-if-email-exists/commit/b451a1e93a6ccf025c78d56dee7439ad607c8507))


### Bug Fixes

* Use std::default for deriving ([#1015](https://github.com/amaurymartiny/check-if-email-exists/issues/1015)) ([03720f0](https://github.com/amaurymartiny/check-if-email-exists/commit/03720f027fd68d5ea5ae538aa567a621f4a65fe3))

### [0.8.25](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.24...v0.8.25) (2021-10-05)


### Features

* Add possibility to set SMTP port ([#985](https://github.com/amaurymartiny/check-if-email-exists/issues/985)) ([cdabdf8](https://github.com/amaurymartiny/check-if-email-exists/commit/cdabdf80e858908d6c33e1273dfdc1fef0f78d35))


### Bug Fixes

* Use async_std_resolver::resolver_from_system_conf ([#982](https://github.com/amaurymartiny/check-if-email-exists/issues/982)) ([376c3b0](https://github.com/amaurymartiny/check-if-email-exists/commit/376c3b0d4743ccc60a1df2a9fa3e9f2f5cd68178))
* Use TLS when available ([#964](https://github.com/amaurymartiny/check-if-email-exists/issues/964)) ([aed11d2](https://github.com/amaurymartiny/check-if-email-exists/commit/aed11d2e15b6b7688ecaf856824ca6effbb5d21b))

### [0.8.24](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.23...v0.8.24) (2021-07-03)


### Features

* Add `CheckEmailInput` setter `set_` prefix to differentiate with accessing fields ([#933](https://github.com/amaurymartiny/check-if-email-exists/issues/933)) ([276f656](https://github.com/amaurymartiny/check-if-email-exists/commit/276f6561e7a98af6415dbd4645d84cbe697b738e))
* Add deprecated warning when running HTTP server ([#943](https://github.com/amaurymartiny/check-if-email-exists/issues/943)) ([e4b1570](https://github.com/amaurymartiny/check-if-email-exists/commit/e4b1570a8be5573f7394a3139f34ab021452cc3a))

### [0.8.23](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.22...v0.8.23) (2021-06-20)


### Bug Fixes

* Add serde (De)Serialize to pub structs ([#931](https://github.com/amaurymartiny/check-if-email-exists/issues/931)) ([949475d](https://github.com/amaurymartiny/check-if-email-exists/commit/949475dee4a1ed96e873688e7432c702eb30af62))

### [0.8.22](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.21...v0.8.22) (2021-03-31)


This is an empty release just to re-run the CI process for building binaries.

### [0.8.21](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.20...v0.8.21) (2021-03-31)


This is an empty release just to re-run the CI process for building binaries.

### [0.8.20](https://github.com/amaurymartiny/check-if-email-exists/compare/v0.8.19...v0.8.20) (2021-03-30)


### Updates

* This release only bumps versions of dependencies, and does not introduce any bugfix or improvements.

### [0.8.19](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.18...v0.8.19) (2021-01-10)


### Features

* Consider CLI config parameters in HTTP request checks ([#827](https://github.com/reacherhq/check-if-email-exists/issues/827)) ([88b751a](https://github.com/reacherhq/check-if-email-exists/commit/88b751a17f4367c990e8a54661e3872898afd10f))


### Bug Fixes

* Reconnect auto-closed SMTP connections by foreign server ([#825](https://github.com/reacherhq/check-if-email-exists/issues/825)) ([01ccf0d](https://github.com/reacherhq/check-if-email-exists/commit/01ccf0d2363475d486bb9827e3e3b9d6954bc032))

### [0.8.18](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.17...v0.8.18) (2021-01-07)


### Bug Fixes

* Check deliverability using successful response code instead of message parsing ([#822](https://github.com/reacherhq/check-if-email-exists/issues/822)) ([39d0ecd](https://github.com/reacherhq/check-if-email-exists/commit/39d0ecdeaf078dce5cdb59cba95ab9e02bce11ee))

### [0.8.17](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.16...v0.8.17) (2021-01-05)


### Bug Fixes

* Add better checks for existing mailboxes ([#819](https://github.com/reacherhq/check-if-email-exists/issues/819)) ([9f88d01](https://github.com/reacherhq/check-if-email-exists/commit/9f88d01fad2c8de898aa35645bab95a14a147393))

### [0.8.16](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.15...v0.8.16) (2020-12-07)


### Features

* Add proxy_host and proxy_port info to HTTP ([#770](https://github.com/reacherhq/check-if-email-exists/issues/770)) ([123f431](https://github.com/reacherhq/check-if-email-exists/commit/123f431e10e90339e030783582d6e4c4919c1a33))

### [0.8.15](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.14...v0.8.15) (2020-11-11)


### Bug Fixes

* Don't check inputted email if catch-all ([#714](https://github.com/reacherhq/check-if-email-exists/issues/714)) ([5129dd1](https://github.com/reacherhq/check-if-email-exists/commit/5129dd1374d3ef93db632f6d7e140e3ce69369b2))
* Fix 'reached the type-length limit while instantiating' ([#665](https://github.com/reacherhq/check-if-email-exists/issues/665)) ([fa040fd](https://github.com/reacherhq/check-if-email-exists/commit/fa040fda8b16ca4d540829ee72f9d7b07ef77fdd))

### [0.8.14](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.13...v0.8.14) (2020-09-24)


### Features

* Add optional timeout on smtp verification ([#611](https://github.com/reacherhq/check-if-email-exists/issues/611)) ([c70de7d](https://github.com/reacherhq/check-if-email-exists/commit/c70de7dcac1811596c78e14888a4258e9db408ed))


### Bug Fixes

* Add more known errors for invalid email ([#543](https://github.com/reacherhq/check-if-email-exists/issues/543)) ([ad209c7](https://github.com/reacherhq/check-if-email-exists/commit/ad209c7ecb3f5aa466f31e293a05734b5edf5f6a))

### [0.8.13](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.12...v0.8.13) (2020-08-04)


### Bug Fixes

* **ci:** Put lto flag in cargo.toml ([#531](https://github.com/reacherhq/check-if-email-exists/issues/531)) ([00cbc1f](https://github.com/reacherhq/check-if-email-exists/commit/00cbc1fd46743c7579809a09b3897259213af496))

### [0.8.12](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.11...v0.8.12) (2020-08-04)


### Bug Fixes

* Add "recipient address accepted" check ([#489](https://github.com/reacherhq/check-if-email-exists/issues/489)) ([5d1e72a](https://github.com/reacherhq/check-if-email-exists/commit/5d1e72ae165f335ab97a96c3806e3293289187a2))
* http request body to use `to_emails` ([#502](https://github.com/reacherhq/check-if-email-exists/issues/502)) ([36aed56](https://github.com/reacherhq/check-if-email-exists/commit/36aed567cf705ef8d20489b2275e3d8ba58b75bb))

### [0.8.11](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.10...v0.8.11) (2020-07-11)


### Bug Fixes

* Add "Invalid email address" check ([#471](https://github.com/reacherhq/check-if-email-exists/issues/471)) ([3b03617](https://github.com/reacherhq/check-if-email-exists/commit/3b03617b81a1f9f6bc1bc6edc8c5d6d9f87eabbb))
* Add possibility to use proxy in Yahoo API request ([#472](https://github.com/reacherhq/check-if-email-exists/issues/472)) ([aafcedf](https://github.com/reacherhq/check-if-email-exists/commit/aafcedf9b9135a6550e7aa2da5d7ca5898da9b53))

### [0.8.10](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.9...v0.8.10) (2020-07-04)


### Bug Fixes

* Correct message parsing for "receiving at a rate" error ([#462](https://github.com/reacherhq/check-if-email-exists/issues/462)) ([4b31706](https://github.com/reacherhq/check-if-email-exists/commit/4b31706228a6e81852505ec21a0f70d5472b1385))

### [0.8.9](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.8...v0.8.9) (2020-07-04)


### Features

* Make using Yahoo API optional ([#460](https://github.com/reacherhq/check-if-email-exists/issues/460)) ([1e42f0a](https://github.com/reacherhq/check-if-email-exists/commit/1e42f0abef27dcea9a467f677ef9a080a3cc0f18))

### [0.8.8](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.7...v0.8.8) (2020-06-28)


### Bug Fixes

* Add debug logs for Yahoo ([e534670](https://github.com/reacherhq/check-if-email-exists/commit/e53467006f9fa435993ea58b1753ce5baa059d2a))

### [0.8.7](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.6...v0.8.7) (2020-06-28)


### Bug Fixes

* Add "recipient unknown" check ([#446](https://github.com/reacherhq/check-if-email-exists/issues/446)) ([deca071](https://github.com/reacherhq/check-if-email-exists/commit/deca071583e34bb9c5836d5238dd51975f827cdc))

### [0.8.6](https://github.com/reacherhq/check-if-email-exists/compare/v0.8.5...v0.8.6) (2020-06-28)


### Bug Fixes

* Add additional error check for undeliverable ([#374](https://github.com/reacherhq/check-if-email-exists/issues/374)) ([e52a8f0](https://github.com/reacherhq/check-if-email-exists/commit/e52a8f0941fd53c9b087e6e59a7018d11af72dff))
* Use HTTP requests to verify Yahoo emails ([#412](https://github.com/reacherhq/check-if-email-exists/issues/412)) ([5fad57d](https://github.com/reacherhq/check-if-email-exists/commit/5fad57d88ef92d65c7d493cdcb45eff347d6a286))

### [0.8.5](https://github.com/reacherhq/check_if_email_exists/compare/v0.8.4...v0.8.5) (2020-05-21)


### Features

* Expose misc, syntax, mx, smtp modules ([#373](https://github.com/reacherhq/check_if_email_exists/issues/373)) ([7c1d741](https://github.com/reacherhq/check_if_email_exists/commit/7c1d741f00b3a807b190140a1d91a42bce29470c))

### [0.8.4](https://github.com/reacherhq/check_if_email_exists/compare/v0.8.3...v0.8.4) (2020-05-19)


### Bug Fixes

* Fix is_reachable with wrong syntax ([#352](https://github.com/reacherhq/check_if_email_exists/issues/352)) ([b0f0209](https://github.com/reacherhq/check_if_email_exists/commit/b0f02090edc0bb8947ab826415fa3bf8b5db55f0))

### [0.8.3](https://github.com/reacherhq/check_if_email_exists/compare/v0.8.2...v0.8.3) (2020-05-12)


### Bug Fixes

* Lowercase Reachable enum variants ([#351](https://github.com/reacherhq/check_if_email_exists/issues/351)) ([b88c20e](https://github.com/reacherhq/check_if_email_exists/commit/b88c20ef5bc947ecd8cc646a9e6c583df0bef4d7))

### [0.8.2](https://github.com/reacherhq/check_if_email_exists/compare/v0.8.1...v0.8.2) (2020-05-12)


### Features

* Add `is_reachable` top field ([#350](https://github.com/reacherhq/check_if_email_exists/issues/350)) ([e7abb17](https://github.com/reacherhq/check_if_email_exists/commit/e7abb17ef29610fbe9210f42830c0ba02bb130b7))
* Detect role-based accounts ([#348](https://github.com/reacherhq/check_if_email_exists/issues/348)) ([7c612fd](https://github.com/reacherhq/check_if_email_exists/commit/7c612fda110729ece094d0b022db05fa4e6b27b4))


### Bug Fixes

* Add "Unknown user" smtp error check ([#347](https://github.com/reacherhq/check_if_email_exists/issues/347)) ([47eb578](https://github.com/reacherhq/check_if_email_exists/commit/47eb5780f692f54aadf264b107996bb2d1a31a56))
* Add more error strings matching ([#323](https://github.com/reacherhq/check_if_email_exists/issues/323)) ([f5392d4](https://github.com/reacherhq/check_if_email_exists/commit/f5392d4befcee6e4d935e1585066eae3b57ec6fa))

### [0.8.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.8.0...v0.8.1) (2020-05-09)


### Bug Fixes

* Lowercase the error string before matching ([#321](https://github.com/reacherhq/check_if_email_exists/issues/321)) ([d983b2f](https://github.com/reacherhq/check_if_email_exists/commit/d983b2fe960ed46c4bd03c55b39d7ea58be5124f))

## [0.8.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.7.1...v0.8.0) (2020-05-08)


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

* Rename valid_format to is_valid_syntax ([#288](https://github.com/reacherhq/check_if_email_exists/issues/288)) ([eae6482](https://github.com/reacherhq/check_if_email_exists/commit/eae64821c31d0193f77d9137ec4e7d6131f91ccb))


* Rename main function to `check_email` ([#319](https://github.com/reacherhq/check_if_email_exists/issues/319)) ([bd12b7d](https://github.com/reacherhq/check_if_email_exists/commit/bd12b7dbbd6c090fcdf80e3d6bbe475cd1d82b9a))

### [0.7.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.7.0...v0.7.1) (2020-04-14)


### Features

* Add possibility to verify emails via proxy ([#286](https://github.com/reacherhq/check_if_email_exists/issues/286)) ([a0ab93f](https://github.com/reacherhq/check_if_email_exists/commit/a0ab93fde5105d594a8280b942d337ff76fbb517))

## [0.7.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.6.7...v0.7.0) (2020-03-26)


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

* Use builder pattern for EmailInput ([#254](https://github.com/reacherhq/check_if_email_exists/issues/254)) ([0c85d36](https://github.com/reacherhq/check_if_email_exists/commit/0c85d36cdccb37d8da9566f7e7baf5dbbd266740))

### [0.6.7](https://github.com/reacherhq/check_if_email_exists/compare/v0.6.6...v0.6.7) (2020-03-20)

### [0.6.6](https://github.com/reacherhq/check_if_email_exists/compare/v0.6.1...v0.6.6) (2020-03-01)


### Bug Fixes

* Allow http to listen to $PORT env variable ([#215](https://github.com/reacherhq/check_if_email_exists/issues/215)) ([3b0c262](https://github.com/reacherhq/check_if_email_exists/commit/3b0c262763bc9d52855ced90aa2a435a97d35d8b))

### [0.6.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.6.0...v0.6.1) (2020-02-18)


### Features

* Add --http-host flag to CLI ([#197](https://github.com/reacherhq/check_if_email_exists/issues/197)) ([55657b2](https://github.com/reacherhq/check_if_email_exists/commit/55657b251fcc22fad2ae53da4f62a017ff01d035))

## [0.6.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.5.0...v0.6.0) (2019-12-01)


### ⚠ BREAKING CHANGES

* - The `is_disposable` subfield has been moved from the `mx` field to a separate `misc` field

### Features

* Add a HTTP server behind the `--http` flag ([#85](https://github.com/reacherhq/check_if_email_exists/issues/85)) ([d8b733e](https://github.com/reacherhq/check_if_email_exists/commit/d8b733e5a571c512644b34219b5f2dfd9dc717b3))
* Add Dockerfile & `x86_64-unknown-linux-musl` target ([#86](https://github.com/reacherhq/check_if_email_exists/issues/86)) ([cba1241](https://github.com/reacherhq/check_if_email_exists/commit/cba124110be04d7febfeab68a6b825197b3aa1fb))

# [0.5.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.4.0...v0.5.0) (2019-11-16)


### Code Refactoring

* Use futures ([#78](https://github.com/reacherhq/check_if_email_exists/issues/78)) ([0e1f6b0](https://github.com/reacherhq/check_if_email_exists/commit/0e1f6b0))


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



# [0.4.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.3.2...v0.4.0) (2019-09-30)


### Features

* Add disposable email check ([#64](https://github.com/reacherhq/check_if_email_exists/issues/64)) ([1b2cea3](https://github.com/reacherhq/check_if_email_exists/commit/1b2cea3))


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



## [0.3.2](https://github.com/reacherhq/check_if_email_exists/compare/v0.3.1...v0.3.2) (2019-09-26)


### Bug Fixes

* **core:** SyntaxError also is type & message ([#60](https://github.com/reacherhq/check_if_email_exists/issues/60)) ([996633b](https://github.com/reacherhq/check_if_email_exists/commit/996633b))



## [0.3.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.3.0...v0.3.1) (2019-09-26)


### Bug Fixes

* Don't use virtual workspace, be able to build ([#59](https://github.com/reacherhq/check_if_email_exists/issues/59)) ([6c93893](https://github.com/reacherhq/check_if_email_exists/commit/6c93893))



# [0.3.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.2.3...v0.3.0) (2019-09-26)


### Features

* New error JSON format ([#56](https://github.com/reacherhq/check_if_email_exists/issues/56)) ([fec4315](https://github.com/reacherhq/check_if_email_exists/commit/fec4315))
* Output JSON information with CLI ([#53](https://github.com/reacherhq/check_if_email_exists/issues/53)) ([1d026d5](https://github.com/reacherhq/check_if_email_exists/commit/1d026d5))
* Return Result<EmailDetails> instead of Result<bool>, with much more details ([#23](https://github.com/reacherhq/check_if_email_exists/issues/23)) ([39b13f5](https://github.com/reacherhq/check_if_email_exists/commit/39b13f5))



## [0.2.3](https://github.com/reacherhq/check_if_email_exists/compare/v0.2.2...v0.2.3) (2019-05-09)


### Bug Fixes

* Update version to correct version in cli ([992777c](https://github.com/reacherhq/check_if_email_exists/commit/992777c))



## [0.2.2](https://github.com/reacherhq/check_if_email_exists/compare/v0.2.1...v0.2.2) (2019-05-09)


### Bug Fixes

* Fix travis and appveyor to build binaries ([f743e67](https://github.com/reacherhq/check_if_email_exists/commit/f743e67))



## [0.2.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.2.0...v0.2.1) (2019-05-09)


### Bug Fixes

* Refactor app to make travis build binaries ([#17](https://github.com/reacherhq/check_if_email_exists/issues/17)) ([9616ef5](https://github.com/reacherhq/check_if_email_exists/commit/9616ef5))



# [0.2.0](https://github.com/reacherhq/check_if_email_exists/compare/v0.1.1...v0.2.0) (2019-05-09)


### Features

* Add serverless function ([#15](https://github.com/reacherhq/check_if_email_exists/issues/15)) ([532c4eb](https://github.com/reacherhq/check_if_email_exists/commit/532c4eb))
* Return Option<bool> instead of bool ([#13](https://github.com/reacherhq/check_if_email_exists/issues/13)) ([2aef345](https://github.com/reacherhq/check_if_email_exists/commit/2aef345))



## [0.1.1](https://github.com/reacherhq/check_if_email_exists/compare/v0.1.0...v0.1.1) (2018-12-29)


# 0.1.0 (2018-12-29)


### Features

* Change codebase to Rust ([#7](https://github.com/reacherhq/check_if_email_exists/pull/7)) ([05569e4](https://github.com/reacherhq/check_if_email_exists/commit/05569e4900b4467fa6d7f03086343fac753fe4ad))
