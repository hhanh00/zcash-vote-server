# Changelog

## [1.3.0](https://github.com/hhanh00/zcash-vote-server/compare/v1.2.2...v1.3.0) (2026-04-07)


### Features

* add -q quit command flag ([#53](https://github.com/hhanh00/zcash-vote-server/issues/53)) ([a4077f0](https://github.com/hhanh00/zcash-vote-server/commit/a4077f08fd37861e77291a1759af5d1940099ec6))
* add ci ([0a59f24](https://github.com/hhanh00/zcash-vote-server/commit/0a59f243c40b96885da5e98ee9d3f020f5f281f3))
* add election id to tables ([#6](https://github.com/hhanh00/zcash-vote-server/issues/6)) ([dc2c1aa](https://github.com/hhanh00/zcash-vote-server/commit/dc2c1aad0bb398ef56e4c94dbe438ee9e5743637))
* add github action release ([#29](https://github.com/hhanh00/zcash-vote-server/issues/29)) ([44dbfda](https://github.com/hhanh00/zcash-vote-server/commit/44dbfda6df35471429390a2ca8cb3a2aabeb8c5b))
* add pre-check ([#22](https://github.com/hhanh00/zcash-vote-server/issues/22)) ([752f44f](https://github.com/hhanh00/zcash-vote-server/commit/752f44f837bdcde9e830ed58c5935a4c2c8679c7))
* add project files ([e58da5a](https://github.com/hhanh00/zcash-vote-server/commit/e58da5aca0c690a32d7d4be3cba4cbf422366bd8))
* add route get /election/id ([#3](https://github.com/hhanh00/zcash-vote-server/issues/3)) ([226fccf](https://github.com/hhanh00/zcash-vote-server/commit/226fccf7ae8333a2e67d2fb7bfe0ffa1b8660bdb))
* add tls support ([#18](https://github.com/hhanh00/zcash-vote-server/issues/18)) ([1341076](https://github.com/hhanh00/zcash-vote-server/commit/134107677a6b0ad8f540c71caabbd38fdb2c2237))
* async mode ([#55](https://github.com/hhanh00/zcash-vote-server/issues/55)) ([9342be0](https://github.com/hhanh00/zcash-vote-server/commit/9342be06e3483306eb7768ef6ef8b30d9d414256))
* ballot verification (signatures, zkp) - not anchors ([#7](https://github.com/hhanh00/zcash-vote-server/issues/7)) ([8ddacd1](https://github.com/hhanh00/zcash-vote-server/commit/8ddacd1892f12d8ea2a92f446ed1ec3afef73d89))
* check ballot roots ([#8](https://github.com/hhanh00/zcash-vote-server/issues/8)) ([9b291c7](https://github.com/hhanh00/zcash-vote-server/commit/9b291c75e5975ee00357abf6c816ef2b1ce27c4e))
* check election id hash ([#27](https://github.com/hhanh00/zcash-vote-server/issues/27)) ([f332679](https://github.com/hhanh00/zcash-vote-server/commit/f332679db931ecfd330ba06f2d8340d92c202fb1))
* comet bft ([#21](https://github.com/hhanh00/zcash-vote-server/issues/21)) ([fedbc6c](https://github.com/hhanh00/zcash-vote-server/commit/fedbc6cbac4a966e3d780088ece1209d9fdaf744))
* db setup ([#1](https://github.com/hhanh00/zcash-vote-server/issues/1)) ([6fa0489](https://github.com/hhanh00/zcash-vote-server/commit/6fa0489ca0f0884da96e2da08eaba44c5a4ef3e5))
* get ballot by height & get number of ballots ([#9](https://github.com/hhanh00/zcash-vote-server/issues/9)) ([fa15134](https://github.com/hhanh00/zcash-vote-server/commit/fa15134e92256dd1667d4beee8d33ca373496e85))
* incrementally add cmx and recompute new cmx_root ([#5](https://github.com/hhanh00/zcash-vote-server/issues/5)) ([1192105](https://github.com/hhanh00/zcash-vote-server/commit/1192105cfd701f91a01d7b7e39dc228ab23511e4))
* open-close election ([#15](https://github.com/hhanh00/zcash-vote-server/issues/15)) ([5268161](https://github.com/hhanh00/zcash-vote-server/commit/5268161499ef3368d0bb1b135fd89df48eb0ef63))
* remove id from election json ([#28](https://github.com/hhanh00/zcash-vote-server/issues/28)) ([8b92741](https://github.com/hhanh00/zcash-vote-server/commit/8b92741f377a5e0ab3f29d87dce82bd25409821f))
* return 500 on validation error ([#12](https://github.com/hhanh00/zcash-vote-server/issues/12)) ([89d7669](https://github.com/hhanh00/zcash-vote-server/commit/89d7669a09e25561b78b4b82da3e8bd98fb850bf))
* save ballot to db & compute new cmx root ([#4](https://github.com/hhanh00/zcash-vote-server/issues/4)) ([a8524b1](https://github.com/hhanh00/zcash-vote-server/commit/a8524b1df00aa8b721af1f874bd7b352be1af716))
* save cmx frontiers in db ([#11](https://github.com/hhanh00/zcash-vote-server/issues/11)) ([637dcda](https://github.com/hhanh00/zcash-vote-server/commit/637dcdab4641160ceb9cdb3ad33d1c9341e17029))
* save elections from data dir to db ([#2](https://github.com/hhanh00/zcash-vote-server/issues/2)) ([b408c10](https://github.com/hhanh00/zcash-vote-server/commit/b408c107ace49df7a48e26ee65fb09b3788493e8))
* update version ([#34](https://github.com/hhanh00/zcash-vote-server/issues/34)) ([e825611](https://github.com/hhanh00/zcash-vote-server/commit/e8256118920f6d8a70f3eae8e509c41a14e5374d))


### Bug Fixes

* add release please ([078ccc4](https://github.com/hhanh00/zcash-vote-server/commit/078ccc400789f322d1bc2c8fcdb430c64c00b3e1))
* app_state must be deterministic ([#49](https://github.com/hhanh00/zcash-vote-server/issues/49)) ([64d24aa](https://github.com/hhanh00/zcash-vote-server/commit/64d24aa58f9f23bac8a9335670fb91e78ed5a934))
* binary not added to release ([#30](https://github.com/hhanh00/zcash-vote-server/issues/30)) ([eb0ebf7](https://github.com/hhanh00/zcash-vote-server/commit/eb0ebf7d522f41cfae8caa1bd36883db50f50b4d))
* build for musl ([015466b](https://github.com/hhanh00/zcash-vote-server/commit/015466b554ed9bfe0a7ff0e714af74e3c55831c9))
* check/propose/finalize/commit flow ([#33](https://github.com/hhanh00/zcash-vote-server/issues/33)) ([d9304a3](https://github.com/hhanh00/zcash-vote-server/commit/d9304a392e9422b55994b5cfd42e3139744caf8d))
* command handler loop should not exit on error ([#35](https://github.com/hhanh00/zcash-vote-server/issues/35)) ([94f12fa](https://github.com/hhanh00/zcash-vote-server/commit/94f12fa9ec8fbea421ae5dc0045ee15d21891afe))
* do not clear the entire mempool on new blocks ([#32](https://github.com/hhanh00/zcash-vote-server/issues/32)) ([b9c735f](https://github.com/hhanh00/zcash-vote-server/commit/b9c735f8d705b8f924373e209bd62715d22c99e1))
* error when no transaction on going ([#58](https://github.com/hhanh00/zcash-vote-server/issues/58)) ([d2c023c](https://github.com/hhanh00/zcash-vote-server/commit/d2c023c83e6bd3e511f0a61c44485bb4b24b2305))
* fix clippy warnings ([#17](https://github.com/hhanh00/zcash-vote-server/issues/17)) ([7747108](https://github.com/hhanh00/zcash-vote-server/commit/77471084ac7472a01a0f82c4f973ac500e0dc38b))
* increase timeout ([#59](https://github.com/hhanh00/zcash-vote-server/issues/59)) ([3268b94](https://github.com/hhanh00/zcash-vote-server/commit/3268b940d5d46f7c26373223e92fb2c1b4d4be01))
* install musl tools ([#47](https://github.com/hhanh00/zcash-vote-server/issues/47)) ([e750a32](https://github.com/hhanh00/zcash-vote-server/commit/e750a32dda1c70d8e03cdd03be4565c694452946))
* missing create_if_missing ([#57](https://github.com/hhanh00/zcash-vote-server/issues/57)) ([2178d43](https://github.com/hhanh00/zcash-vote-server/commit/2178d438de1188e879140cfe56a063240edcc0f1))
* **vote-server:** fix non determinism apphash crashes caused by sorting by the wrong ID in sql ([#61](https://github.com/hhanh00/zcash-vote-server/issues/61)) ([76374dd](https://github.com/hhanh00/zcash-vote-server/commit/76374dd62f93d71fc89c201554316f17a7a7a52a))
* zip -&gt; tgz ([#31](https://github.com/hhanh00/zcash-vote-server/issues/31)) ([c03f228](https://github.com/hhanh00/zcash-vote-server/commit/c03f2288f65cc066e4f85c04a0a827d5ab5c892a))

## [1.1.0](https://github.com/hhanh00/zcash-vote-server/compare/v1.0.4...v1.1.0) (2025-11-01)


### Features

* add -q quit command flag ([#53](https://github.com/hhanh00/zcash-vote-server/issues/53)) ([a4077f0](https://github.com/hhanh00/zcash-vote-server/commit/a4077f08fd37861e77291a1759af5d1940099ec6))

## [1.0.4](https://github.com/hhanh00/zcash-vote-server/compare/v1.0.3...v1.0.4) (2025-10-31)


### Bug Fixes

* app_state must be deterministic ([#49](https://github.com/hhanh00/zcash-vote-server/issues/49)) ([64d24aa](https://github.com/hhanh00/zcash-vote-server/commit/64d24aa58f9f23bac8a9335670fb91e78ed5a934))

## [1.0.3](https://github.com/hhanh00/zcash-vote-server/compare/v1.0.2...v1.0.3) (2025-10-30)


### Bug Fixes

* add release please ([078ccc4](https://github.com/hhanh00/zcash-vote-server/commit/078ccc400789f322d1bc2c8fcdb430c64c00b3e1))
* build for musl ([015466b](https://github.com/hhanh00/zcash-vote-server/commit/015466b554ed9bfe0a7ff0e714af74e3c55831c9))
* install musl tools ([#47](https://github.com/hhanh00/zcash-vote-server/issues/47)) ([e750a32](https://github.com/hhanh00/zcash-vote-server/commit/e750a32dda1c70d8e03cdd03be4565c694452946))

## [1.0.2](https://github.com/hhanh00/zcash-vote-server/compare/v1.0.1...v1.0.2) (2025-04-02)


### Bug Fixes

* command handler loop should not exit on error ([#35](https://github.com/hhanh00/zcash-vote-server/issues/35)) ([94f12fa](https://github.com/hhanh00/zcash-vote-server/commit/94f12fa9ec8fbea421ae5dc0045ee15d21891afe))
