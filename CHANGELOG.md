# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## 1.1.0 (2020-12-14)


### Features

* **audit:** add scheduled security audit scan ([7bbb5c4](https://github.com/Yoga07/sn_transfers/commit/7bbb5c43fafd9666364b2a9e308d7c0c9f6824d8))
* **genesis:** expose genesis generator ([7f48117](https://github.com/Yoga07/sn_transfers/commit/7f4811752dfb2a18b27fe082b99045ed8d880a00))
* **no PublicId:** updated for data_type changes for No PublicIds ([e2474c6](https://github.com/Yoga07/sn_transfers/commit/e2474c6d01b8c4c9e05245dfa9c9e0052110aac7))


### Bug Fixes

* use master pubkey as id for multisig wallet ([5fd6715](https://github.com/Yoga07/sn_transfers/commit/5fd6715ac74037750c128752f3624a5c5f0f4945))
* wrap replica counter in option ([7dbe918](https://github.com/Yoga07/sn_transfers/commit/7dbe918190a3f3a8813bb4b32c4dea6d6f2d8ecd))
* **validation:** add case zero ([5b3cd71](https://github.com/Yoga07/sn_transfers/commit/5b3cd71c82a5f6ead874451dd97f75319979b73f))
* add validation of id exists on propagated ([b1750af](https://github.com/Yoga07/sn_transfers/commit/b1750afb595b7fc36e27405334b12a1703257c82))
* align transfer-split and actor model changes ([dc30b7d](https://github.com/Yoga07/sn_transfers/commit/dc30b7d82acb2711b61db724675bb8f84edb1ad9))
* ignore sig validation for simulated payouts ([45a21a1](https://github.com/Yoga07/sn_transfers/commit/45a21a124e343666336756878f9cbae89456cff6))
* verify correct signature for credit ([c7b51de](https://github.com/Yoga07/sn_transfers/commit/c7b51de35405516c0a3542cc4d373b8094c5be2a))
* **0:** don't allow sending of no money ([fffbf8c](https://github.com/Yoga07/sn_transfers/commit/fffbf8ca19debfcbf36a212e184a273ae4ba1830))
* **actor:** move mutation to apply fn ([b51ab31](https://github.com/Yoga07/sn_transfers/commit/b51ab31746af06241107de932f7bab236e004294))
* **all:** remove all unwraps from library and test code ([ee0520a](https://github.com/Yoga07/sn_transfers/commit/ee0520a1f8ad018c0e7d743762bb9a35880406dd))

### [0.2.4](https://github.com/maidsafe/sn_transfers/compare/v0.2.3...v0.2.4) (2020-11-24)


### Bug Fixes

* **all:** remove all unwraps from library and test code ([ee0520a](https://github.com/maidsafe/sn_transfers/commit/ee0520a1f8ad018c0e7d743762bb9a35880406dd))

### [0.2.3](https://github.com/maidsafe/sn_transfers/compare/v0.2.2...v0.2.3) (2020-11-23)

### [0.2.2](https://github.com/maidsafe/sn_transfers/compare/v0.2.1...v0.2.2) (2020-10-27)


### Features

* **no PublicId:** updated for data_type changes for No PublicIds ([e2474c6](https://github.com/maidsafe/sn_transfers/commit/e2474c6d01b8c4c9e05245dfa9c9e0052110aac7))

### [0.2.1](https://github.com/maidsafe/sn_transfers/compare/v0.2.0...v0.2.1) (2020-10-20)


### Bug Fixes

* **0:** don't allow sending of no money ([fffbf8c](https://github.com/maidsafe/sn_transfers/commit/fffbf8ca19debfcbf36a212e184a273ae4ba1830))
* **actor:** move mutation to apply fn ([b51ab31](https://github.com/maidsafe/sn_transfers/commit/b51ab31746af06241107de932f7bab236e004294))

### [0.2.0](https://github.com/maidsafe/sn_transfers/compare/v0.1.0...v0.2.0) (2020-09-03)

* Update crate name to sn_transfers.
* Expose genesis generator.
* Add initial infusion of money.
* Check against previous 'next' count when applying RegisteredTransfers.
* Support checking SectionProofChain in safe_vaults.
* Add scheduled security audit scan.
* Update simulated payout funcs to credit/debit correct pk.
* Add GetReplicaKeys.
* Add simulated debitting APIs to replica.
* Refactor and fix simulated-payouts APIs.
* Add simulated-payouts feature and include testing API to Replica.
* Fix received debits logic.
* Accumulate remote credits.
* Add peer logic and sig validations.

### [0.1.0](https://github.com/maidsafe/sn_transfers/compare/v0.1.0...v0.1.0) (2020-05-19)

* Initial implementation.
