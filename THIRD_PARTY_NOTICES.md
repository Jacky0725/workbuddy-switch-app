# WorkBuddy Manager — Third-party notices

This is an independent experimental fork, with no service-provider endorsement. The inventory below retains its baseline package versions; this notice was updated on 2026-10-22.

Copyright and full permission notices are retained in the accompanying `licenses/` directory and root `LICENSE`.
Source: workbuddy-switch v0.1.58, commit 9eb3f2a21ff3c3c85fef2f89e693c48792050ba1 (MIT, Copyright (c) 2026 wb-switch).
Helper was consulted as an architectural reference; no Go executable or international simulated-activity implementation is bundled.
The optional Agent Companion runtime and its character artwork are disabled and are not included as installer resources. Its Rust plugin dependency is conservatively listed below.
The original geometric WM product/tray artwork is created for this fork. Third-party product logos are replaced by text marks in the compiled UI.
The Bricolage Grotesque font uses OFL-1.1; its full license is retained. Lucide React uses ISC.
MPL-2.0 crate corresponding source is retained under each crate’s `corresponding-source/` directory, unmodified.
The upstream indexmap 1.9.3 build-script patch is retained in project source (`vendor-patched/indexmap`), with its MIT/Apache notices.
react-remove-scroll-bar 2.3.8 declares MIT in its integrity-verified npm release. All 26 non-manifest files are byte-identical to the official 2.3.7 release; only the version and react-style-singleton dependency range differ. The upstream change from the 2.3.7 release commit to 8ca9ba5ea52de03308fe8ced94f7b159a44d28ff only adds the retained MIT LICENSE. This verifies the published code payload’s license correspondence without claiming access to the unavailable 2.3.8 gitHead or a reproducible source build. See [provenance](licenses/npm/react-remove-scroll-bar-2.3.8/PROVENANCE.json).
This inventory intentionally includes build dependencies; it is not a symbol-level binary linkage report.

| Ecosystem | Package | Version | License | Full text |
| --- | --- | --- | --- | --- |
| rust | wb-switch-core | 0.2.0-alpha.1 | MIT (upstream root LICENSE) | [LICENSE](licenses/rust/wb-switch-core-0.2.0-alpha.1/LICENSE) |
| rust | wb-switch-rust | 0.2.0-alpha.1 | MIT (upstream root LICENSE) | [LICENSE](licenses/rust/wb-switch-rust-0.2.0-alpha.1/LICENSE) |
| rust | indexmap | 1.9.3 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/indexmap-1.9.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/indexmap-1.9.3/LICENSE-MIT) |
| rust | agent-studio-core | 0.1.0 | MIT | [LICENSE](licenses/rust/agent-studio-core-0.1.0/LICENSE) |
| rust | tauri-plugin-agent-studio | 0.1.0 | MIT | [LICENSE](licenses/rust/tauri-plugin-agent-studio-0.1.0/LICENSE) |
| rust | agent-studio-runtime | 0.1.0 | MIT | [LICENSE](licenses/rust/agent-studio-runtime-0.1.0/LICENSE) |
| rust | adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | [LICENSE-0BSD](licenses/rust/adler2-2.0.1/LICENSE-0BSD), [LICENSE-APACHE](licenses/rust/adler2-2.0.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/adler2-2.0.1/LICENSE-MIT) |
| rust | aead | 0.5.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/aead-0.5.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/aead-0.5.2/LICENSE-MIT) |
| rust | aes-gcm | 0.10.3 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/aes-gcm-0.10.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/aes-gcm-0.10.3/LICENSE-MIT) |
| rust | aes | 0.8.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/aes-0.8.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/aes-0.8.4/LICENSE-MIT) |
| rust | ahash | 0.8.12 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ahash-0.8.12/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ahash-0.8.12/LICENSE-MIT) |
| rust | aho-corasick | 1.1.5 | Unlicense OR MIT | [COPYING](licenses/rust/aho-corasick-1.1.5/COPYING), [LICENSE-MIT](licenses/rust/aho-corasick-1.1.5/LICENSE-MIT) |
| rust | alloc-no-stdlib | 2.0.4 | BSD-3-Clause | [LICENSE](licenses/rust/alloc-no-stdlib-2.0.4/LICENSE) |
| rust | alloc-stdlib | 0.2.4 | BSD-3-Clause | [LICENSE-upstream-LICENSE](licenses/rust/alloc-stdlib-0.2.4/LICENSE-upstream-LICENSE) |
| rust | anyhow | 1.0.104 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/anyhow-1.0.104/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/anyhow-1.0.104/LICENSE-MIT) |
| rust | ascii | 1.1.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/ascii-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ascii-1.1.0/LICENSE-MIT) |
| rust | atomic-waker | 1.1.2 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/atomic-waker-1.1.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/atomic-waker-1.1.2/LICENSE-MIT), [LICENSE-THIRD-PARTY](licenses/rust/atomic-waker-1.1.2/LICENSE-THIRD-PARTY) |
| rust | auto-launch | 0.5.0 | MIT | [LICENSE](licenses/rust/auto-launch-0.5.0/LICENSE) |
| rust | autocfg | 1.5.1 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/autocfg-1.5.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/autocfg-1.5.1/LICENSE-MIT) |
| rust | base64 | 0.22.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/base64-0.22.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/base64-0.22.1/LICENSE-MIT) |
| rust | bit-set | 0.8.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/bit-set-0.8.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/bit-set-0.8.0/LICENSE-MIT) |
| rust | bit-vec | 0.8.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/bit-vec-0.8.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/bit-vec-0.8.0/LICENSE-MIT) |
| rust | bitflags | 1.3.2 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/bitflags-1.3.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/bitflags-1.3.2/LICENSE-MIT) |
| rust | bitflags | 2.13.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/bitflags-2.13.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/bitflags-2.13.1/LICENSE-MIT) |
| rust | block-buffer | 0.10.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/block-buffer-0.10.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/block-buffer-0.10.4/LICENSE-MIT) |
| rust | block-padding | 0.3.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/block-padding-0.3.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/block-padding-0.3.3/LICENSE-MIT) |
| rust | brotli-decompressor | 5.0.3 | BSD-3-Clause/MIT | [LICENSE](licenses/rust/brotli-decompressor-5.0.3/LICENSE) |
| rust | brotli | 8.0.4 | BSD-3-Clause AND MIT | [LICENSE.BSD-3-Clause](licenses/rust/brotli-8.0.4/LICENSE.BSD-3-Clause), [LICENSE.MIT](licenses/rust/brotli-8.0.4/LICENSE.MIT) |
| rust | bs58 | 0.5.1 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/bs58-0.5.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/bs58-0.5.1/LICENSE-MIT) |
| rust | byteorder | 1.5.0 | Unlicense OR MIT | [COPYING](licenses/rust/byteorder-1.5.0/COPYING), [LICENSE-MIT](licenses/rust/byteorder-1.5.0/LICENSE-MIT) |
| rust | bytes | 1.12.1 | MIT | [LICENSE](licenses/rust/bytes-1.12.1/LICENSE) |
| rust | camino | 1.2.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/camino-1.2.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/camino-1.2.5/LICENSE-MIT) |
| rust | cargo-platform | 0.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cargo-platform-0.1.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cargo-platform-0.1.9/LICENSE-MIT) |
| rust | cargo_metadata | 0.19.2 | MIT | [LICENSE-MIT](licenses/rust/cargo_metadata-0.19.2/LICENSE-MIT) |
| rust | cargo_toml | 0.22.3 | Apache-2.0 OR MIT | [LICENSE](licenses/rust/cargo_toml-0.22.3/LICENSE) |
| rust | cbc | 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cbc-0.1.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cbc-0.1.2/LICENSE-MIT) |
| rust | cc | 1.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cc-1.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cc-1.4.3/LICENSE-MIT) |
| rust | cfb | 0.7.3 | MIT | [LICENSE](licenses/rust/cfb-0.7.3/LICENSE) |
| rust | cfg-if | 1.0.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cfg-if-1.0.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cfg-if-1.0.4/LICENSE-MIT) |
| rust | cfg_aliases | 0.2.2 | MIT | [LICENSE](licenses/rust/cfg_aliases-0.2.2/LICENSE), [NOTICES.md](licenses/rust/cfg_aliases-0.2.2/NOTICES.md) |
| rust | chacha20 | 0.10.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/chacha20-0.10.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/chacha20-0.10.1/LICENSE-MIT) |
| rust | chrono | 0.4.45 | MIT OR Apache-2.0 | [LICENSE.txt](licenses/rust/chrono-0.4.45/LICENSE.txt) |
| rust | chunked_transfer | 1.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/chunked_transfer-1.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/chunked_transfer-1.5.0/LICENSE-MIT) |
| rust | cipher | 0.4.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cipher-0.4.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cipher-0.4.4/LICENSE-MIT) |
| rust | cookie | 0.18.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cookie-0.18.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cookie-0.18.2/LICENSE-MIT) |
| rust | cpufeatures | 0.2.17 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cpufeatures-0.2.17/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cpufeatures-0.2.17/LICENSE-MIT) |
| rust | cpufeatures | 0.3.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/cpufeatures-0.3.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/cpufeatures-0.3.0/LICENSE-MIT) |
| rust | crc32fast | 1.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/crc32fast-1.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/crc32fast-1.5.0/LICENSE-MIT) |
| rust | crossbeam-channel | 0.5.16 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/crossbeam-channel-0.5.16/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/crossbeam-channel-0.5.16/LICENSE-MIT), [LICENSE-THIRD-PARTY](licenses/rust/crossbeam-channel-0.5.16/LICENSE-THIRD-PARTY) |
| rust | crossbeam-utils | 0.8.22 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/crossbeam-utils-0.8.22/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/crossbeam-utils-0.8.22/LICENSE-MIT) |
| rust | crypto-common | 0.1.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/crypto-common-0.1.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/crypto-common-0.1.7/LICENSE-MIT) |
| rust | cssparser-macros | 0.6.1 | MPL-2.0 | [LICENSE](licenses/rust/cssparser-macros-0.6.1/LICENSE) |
| rust | cssparser | 0.36.0 | MPL-2.0 | [LICENSE](licenses/rust/cssparser-0.36.0/LICENSE) |
| rust | ctor-proc-macro | 0.0.7 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/ctor-proc-macro-0.0.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ctor-proc-macro-0.0.7/LICENSE-MIT) |
| rust | ctor | 0.8.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/ctor-0.8.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ctor-0.8.0/LICENSE-MIT) |
| rust | ctr | 0.9.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ctr-0.9.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ctr-0.9.2/LICENSE-MIT) |
| rust | darling | 0.23.0 | MIT | [LICENSE](licenses/rust/darling-0.23.0/LICENSE) |
| rust | darling_core | 0.23.0 | MIT | [LICENSE](licenses/rust/darling_core-0.23.0/LICENSE) |
| rust | darling_macro | 0.23.0 | MIT | [LICENSE](licenses/rust/darling_macro-0.23.0/LICENSE) |
| rust | data-encoding | 2.11.1 | MIT | [LICENSE](licenses/rust/data-encoding-2.11.1/LICENSE) |
| rust | defmt-macros | 1.1.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/defmt-macros-1.1.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/defmt-macros-1.1.1/LICENSE-MIT) |
| rust | defmt-parser | 1.0.0 | MIT OR Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/defmt-parser-1.0.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/defmt-parser-1.0.0/LICENSE-upstream-LICENSE-MIT) |
| rust | defmt | 1.1.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/defmt-1.1.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/defmt-1.1.1/LICENSE-MIT) |
| rust | deranged | 0.5.8 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/deranged-0.5.8/LICENSE-Apache), [LICENSE-MIT](licenses/rust/deranged-0.5.8/LICENSE-MIT) |
| rust | derive_more-impl | 2.1.1 | MIT | [LICENSE](licenses/rust/derive_more-impl-2.1.1/LICENSE) |
| rust | derive_more | 2.1.1 | MIT | [LICENSE](licenses/rust/derive_more-2.1.1/LICENSE) |
| rust | digest | 0.10.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/digest-0.10.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/digest-0.10.7/LICENSE-MIT) |
| rust | dirs-sys | 0.4.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dirs-sys-0.4.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dirs-sys-0.4.1/LICENSE-MIT) |
| rust | dirs-sys | 0.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dirs-sys-0.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dirs-sys-0.5.0/LICENSE-MIT) |
| rust | dirs | 5.0.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dirs-5.0.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dirs-5.0.1/LICENSE-MIT) |
| rust | dirs | 6.0.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dirs-6.0.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dirs-6.0.0/LICENSE-MIT) |
| rust | displaydoc | 0.2.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/displaydoc-0.2.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/displaydoc-0.2.7/LICENSE-MIT) |
| rust | dom_query | 0.27.0 | MIT | [LICENSE](licenses/rust/dom_query-0.27.0/LICENSE) |
| rust | dpi | 0.1.2 | Apache-2.0 AND MIT | [LICENSE](licenses/rust/dpi-0.1.2/LICENSE), [LICENSE-LIBM-MIT](licenses/rust/dpi-0.1.2/LICENSE-LIBM-MIT) |
| rust | dtoa-short | 0.3.5 | MPL-2.0 | [LICENSE](licenses/rust/dtoa-short-0.3.5/LICENSE) |
| rust | dtoa | 1.0.11 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dtoa-1.0.11/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dtoa-1.0.11/LICENSE-MIT) |
| rust | dtor-proc-macro | 0.0.6 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/dtor-proc-macro-0.0.6/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dtor-proc-macro-0.0.6/LICENSE-MIT) |
| rust | dtor | 0.3.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/dtor-0.3.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dtor-0.3.0/LICENSE-MIT) |
| rust | dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [LICENSE](licenses/rust/dunce-1.0.5/LICENSE) |
| rust | dyn-clone | 1.0.20 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/dyn-clone-1.0.20/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/dyn-clone-1.0.20/LICENSE-MIT) |
| rust | embed-resource | 3.0.11 | MIT | [LICENSE](licenses/rust/embed-resource-3.0.11/LICENSE) |
| rust | equivalent | 1.0.2 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/equivalent-1.0.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/equivalent-1.0.2/LICENSE-MIT) |
| rust | erased-serde | 0.4.10 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/erased-serde-0.4.10/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/erased-serde-0.4.10/LICENSE-MIT) |
| rust | fallible-iterator | 0.3.0 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/fallible-iterator-0.3.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fallible-iterator-0.3.0/LICENSE-MIT) |
| rust | fallible-streaming-iterator | 0.1.9 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/fallible-streaming-iterator-0.1.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fallible-streaming-iterator-0.1.9/LICENSE-MIT) |
| rust | fastrand | 2.5.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/fastrand-2.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fastrand-2.5.0/LICENSE-MIT) |
| rust | fdeflate | 0.3.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/fdeflate-0.3.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fdeflate-0.3.7/LICENSE-MIT) |
| rust | find-msvc-tools | 0.1.11 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/find-msvc-tools-0.1.11/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/find-msvc-tools-0.1.11/LICENSE-MIT) |
| rust | flate2 | 1.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/flate2-1.1.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/flate2-1.1.9/LICENSE-MIT) |
| rust | fnv | 1.0.7 | Apache-2.0 / MIT | [LICENSE-APACHE](licenses/rust/fnv-1.0.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fnv-1.0.7/LICENSE-MIT) |
| rust | foldhash | 0.2.0 | Zlib | [LICENSE](licenses/rust/foldhash-0.2.0/LICENSE) |
| rust | form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/form_urlencoded-1.2.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/form_urlencoded-1.2.2/LICENSE-MIT) |
| rust | fs2 | 0.4.3 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/fs2-0.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/fs2-0.4.3/LICENSE-MIT) |
| rust | futures-channel | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-channel-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-channel-0.3.34/LICENSE-MIT) |
| rust | futures-core | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-core-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-core-0.3.34/LICENSE-MIT) |
| rust | futures-io | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-io-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-io-0.3.34/LICENSE-MIT) |
| rust | futures-macro | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-macro-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-macro-0.3.34/LICENSE-MIT) |
| rust | futures-sink | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-sink-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-sink-0.3.34/LICENSE-MIT) |
| rust | futures-task | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-task-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-task-0.3.34/LICENSE-MIT) |
| rust | futures-util | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/futures-util-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/futures-util-0.3.34/LICENSE-MIT) |
| rust | generic-array | 0.14.7 | MIT | [LICENSE](licenses/rust/generic-array-0.14.7/LICENSE) |
| rust | getrandom | 0.2.17 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/getrandom-0.2.17/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/getrandom-0.2.17/LICENSE-MIT) |
| rust | getrandom | 0.3.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/getrandom-0.3.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/getrandom-0.3.4/LICENSE-MIT) |
| rust | getrandom | 0.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/getrandom-0.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/getrandom-0.4.3/LICENSE-MIT) |
| rust | ghash | 0.5.1 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/ghash-0.5.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ghash-0.5.1/LICENSE-MIT) |
| rust | glob | 0.3.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/glob-0.3.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/glob-0.3.4/LICENSE-MIT) |
| rust | hashbrown | 0.12.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hashbrown-0.12.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hashbrown-0.12.3/LICENSE-MIT) |
| rust | hashbrown | 0.14.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hashbrown-0.14.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hashbrown-0.14.5/LICENSE-MIT) |
| rust | hashbrown | 0.17.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hashbrown-0.17.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hashbrown-0.17.1/LICENSE-MIT) |
| rust | hashlink | 0.9.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hashlink-0.9.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hashlink-0.9.1/LICENSE-MIT) |
| rust | heck | 0.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/heck-0.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/heck-0.5.0/LICENSE-MIT) |
| rust | hex | 0.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hex-0.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hex-0.4.3/LICENSE-MIT) |
| rust | hmac | 0.12.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/hmac-0.12.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/hmac-0.12.1/LICENSE-MIT) |
| rust | html5ever | 0.38.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/html5ever-0.38.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/html5ever-0.38.0/LICENSE-MIT) |
| rust | http-body-util | 0.1.5 | MIT | [LICENSE](licenses/rust/http-body-util-0.1.5/LICENSE) |
| rust | http-body | 1.1.0 | MIT | [LICENSE](licenses/rust/http-body-1.1.0/LICENSE) |
| rust | http | 1.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/http-1.5.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/http-1.5.0/LICENSE-MIT) |
| rust | httparse | 1.10.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/httparse-1.10.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/httparse-1.10.1/LICENSE-MIT) |
| rust | httpdate | 1.0.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/httpdate-1.0.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/httpdate-1.0.3/LICENSE-MIT) |
| rust | hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT | [LICENSE-APACHE](licenses/rust/hyper-rustls-0.27.9/LICENSE-APACHE), [LICENSE-ISC](licenses/rust/hyper-rustls-0.27.9/LICENSE-ISC), [LICENSE-MIT](licenses/rust/hyper-rustls-0.27.9/LICENSE-MIT) |
| rust | hyper-util | 0.1.20 | MIT | [LICENSE](licenses/rust/hyper-util-0.1.20/LICENSE) |
| rust | hyper | 1.11.0 | MIT | [LICENSE](licenses/rust/hyper-1.11.0/LICENSE) |
| rust | ico | 0.5.0 | MIT | [LICENSE](licenses/rust/ico-0.5.0/LICENSE) |
| rust | icu_collections | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_collections-2.3.0/LICENSE) |
| rust | icu_locale_core | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_locale_core-2.3.0/LICENSE) |
| rust | icu_normalizer | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_normalizer-2.3.0/LICENSE) |
| rust | icu_normalizer_data | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_normalizer_data-2.3.0/LICENSE) |
| rust | icu_properties | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_properties-2.3.0/LICENSE) |
| rust | icu_properties_data | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_properties_data-2.3.0/LICENSE) |
| rust | icu_provider | 2.3.0 | Unicode-3.0 | [LICENSE](licenses/rust/icu_provider-2.3.0/LICENSE) |
| rust | ident_case | 1.0.1 | MIT/Apache-2.0 | [LICENSE](licenses/rust/ident_case-1.0.1/LICENSE) |
| rust | idna | 1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/idna-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/idna-1.1.0/LICENSE-MIT) |
| rust | idna_adapter | 1.2.2 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/idna_adapter-1.2.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/idna_adapter-1.2.2/LICENSE-MIT) |
| rust | indexmap | 2.14.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/indexmap-2.14.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/indexmap-2.14.0/LICENSE-MIT) |
| rust | infer | 0.19.0 | MIT | [LICENSE](licenses/rust/infer-0.19.0/LICENSE) |
| rust | inout | 0.1.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/inout-0.1.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/inout-0.1.4/LICENSE-MIT) |
| rust | ipnet | 2.12.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ipnet-2.12.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ipnet-2.12.1/LICENSE-MIT) |
| rust | itoa | 1.0.18 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/itoa-1.0.18/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/itoa-1.0.18/LICENSE-MIT) |
| rust | jiff-core | 0.1.0 | Unlicense OR MIT | [COPYING](licenses/rust/jiff-core-0.1.0/COPYING), [LICENSE-MIT](licenses/rust/jiff-core-0.1.0/LICENSE-MIT) |
| rust | jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT | [COPYING](licenses/rust/jiff-tzdb-platform-0.1.3/COPYING), [LICENSE-MIT](licenses/rust/jiff-tzdb-platform-0.1.3/LICENSE-MIT) |
| rust | jiff-tzdb | 0.1.8 | Unlicense OR MIT | [COPYING](licenses/rust/jiff-tzdb-0.1.8/COPYING), [LICENSE-MIT](licenses/rust/jiff-tzdb-0.1.8/LICENSE-MIT) |
| rust | jiff | 0.2.35 | Unlicense OR MIT | [COPYING](licenses/rust/jiff-0.2.35/COPYING), [LICENSE-MIT](licenses/rust/jiff-0.2.35/LICENSE-MIT) |
| rust | json-patch | 3.0.1 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/json-patch-3.0.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/json-patch-3.0.1/LICENSE-MIT) |
| rust | jsonptr | 0.6.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/jsonptr-0.6.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/jsonptr-0.6.3/LICENSE-MIT) |
| rust | keyboard-types | 0.7.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/keyboard-types-0.7.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/keyboard-types-0.7.0/LICENSE-MIT) |
| rust | libc | 0.2.189 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/libc-0.2.189/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/libc-0.2.189/LICENSE-MIT) |
| rust | libsqlite3-sys | 0.30.1 | MIT | [LICENSE](licenses/rust/libsqlite3-sys-0.30.1/LICENSE) |
| rust | litemap | 0.8.3 | Unicode-3.0 | [LICENSE](licenses/rust/litemap-0.8.3/LICENSE) |
| rust | lock_api | 0.4.14 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/lock_api-0.4.14/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/lock_api-0.4.14/LICENSE-MIT) |
| rust | log | 0.4.33 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/log-0.4.33/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/log-0.4.33/LICENSE-MIT) |
| rust | lru-slab | 0.1.2 | MIT OR Apache-2.0 OR Zlib | [LICENSE-APACHE](licenses/rust/lru-slab-0.1.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/lru-slab-0.1.2/LICENSE-MIT), [LICENSE-ZLIB](licenses/rust/lru-slab-0.1.2/LICENSE-ZLIB) |
| rust | markup5ever | 0.38.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/markup5ever-0.38.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/markup5ever-0.38.0/LICENSE-MIT) |
| rust | memchr | 2.8.3 | Unlicense OR MIT | [COPYING](licenses/rust/memchr-2.8.3/COPYING), [LICENSE-MIT](licenses/rust/memchr-2.8.3/LICENSE-MIT) |
| rust | mime | 0.3.17 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/mime-0.3.17/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/mime-0.3.17/LICENSE-MIT) |
| rust | minisign-verify | 0.2.5 | MIT | [LICENSE](licenses/rust/minisign-verify-0.2.5/LICENSE) |
| rust | miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | [LICENSE](licenses/rust/miniz_oxide-0.8.9/LICENSE), [LICENSE-APACHE.md](licenses/rust/miniz_oxide-0.8.9/LICENSE-APACHE.md), [LICENSE-MIT.md](licenses/rust/miniz_oxide-0.8.9/LICENSE-MIT.md), [LICENSE-ZLIB.md](licenses/rust/miniz_oxide-0.8.9/LICENSE-ZLIB.md) |
| rust | mio | 1.2.2 | MIT | [LICENSE](licenses/rust/mio-1.2.2/LICENSE) |
| rust | muda | 0.19.3 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/muda-0.19.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/muda-0.19.3/LICENSE-MIT), [LICENSE.spdx](licenses/rust/muda-0.19.3/LICENSE.spdx) |
| rust | new_debug_unreachable | 1.0.6 | MIT | [LICENSE-MIT](licenses/rust/new_debug_unreachable-1.0.6/LICENSE-MIT) |
| rust | notify-rust | 4.18.0 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/notify-rust-4.18.0/LICENSE-Apache), [LICENSE-MIT](licenses/rust/notify-rust-4.18.0/LICENSE-MIT) |
| rust | num-conv | 0.2.2 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/num-conv-0.2.2/LICENSE-Apache), [LICENSE-MIT](licenses/rust/num-conv-0.2.2/LICENSE-MIT) |
| rust | num-traits | 0.2.19 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/num-traits-0.2.19/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/num-traits-0.2.19/LICENSE-MIT) |
| rust | once_cell | 1.21.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/once_cell-1.21.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/once_cell-1.21.4/LICENSE-MIT) |
| rust | opaque-debug | 0.3.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/opaque-debug-0.3.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/opaque-debug-0.3.1/LICENSE-MIT) |
| rust | open | 5.4.1 | MIT | [LICENSE.md](licenses/rust/open-5.4.1/LICENSE.md) |
| rust | option-ext | 0.2.0 | MPL-2.0 | [LICENSE.txt](licenses/rust/option-ext-0.2.0/LICENSE.txt) |
| rust | parking_lot | 0.12.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/parking_lot-0.12.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/parking_lot-0.12.5/LICENSE-MIT) |
| rust | parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/parking_lot_core-0.9.12/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/parking_lot_core-0.9.12/LICENSE-MIT) |
| rust | pbkdf2 | 0.12.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/pbkdf2-0.12.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/pbkdf2-0.12.2/LICENSE-MIT) |
| rust | percent-encoding | 2.3.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/percent-encoding-2.3.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/percent-encoding-2.3.2/LICENSE-MIT) |
| rust | phf | 0.13.1 | MIT | [LICENSE](licenses/rust/phf-0.13.1/LICENSE) |
| rust | phf_codegen | 0.13.1 | MIT | [LICENSE](licenses/rust/phf_codegen-0.13.1/LICENSE) |
| rust | phf_generator | 0.13.1 | MIT | [LICENSE](licenses/rust/phf_generator-0.13.1/LICENSE) |
| rust | phf_macros | 0.13.1 | MIT | [LICENSE](licenses/rust/phf_macros-0.13.1/LICENSE) |
| rust | phf_shared | 0.13.1 | MIT | [LICENSE](licenses/rust/phf_shared-0.13.1/LICENSE) |
| rust | pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/pin-project-lite-0.2.17/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/pin-project-lite-0.2.17/LICENSE-MIT) |
| rust | pkg-config | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/pkg-config-0.3.34/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/pkg-config-0.3.34/LICENSE-MIT) |
| rust | plist | 1.10.0 | MIT | [LICENCE](licenses/rust/plist-1.10.0/LICENCE) |
| rust | png | 0.17.16 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/png-0.17.16/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/png-0.17.16/LICENSE-MIT) |
| rust | polyval | 0.6.2 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/polyval-0.6.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/polyval-0.6.2/LICENSE-MIT) |
| rust | potential_utf | 0.1.6 | Unicode-3.0 | [LICENSE](licenses/rust/potential_utf-0.1.6/LICENSE) |
| rust | powerfmt | 0.2.0 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/powerfmt-0.2.0/LICENSE-Apache), [LICENSE-MIT](licenses/rust/powerfmt-0.2.0/LICENSE-MIT) |
| rust | ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ppv-lite86-0.2.21/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ppv-lite86-0.2.21/LICENSE-MIT) |
| rust | precomputed-hash | 0.1.1 | MIT | [LICENSE](licenses/rust/precomputed-hash-0.1.1/LICENSE) |
| rust | proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/proc-macro2-1.0.107/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/proc-macro2-1.0.107/LICENSE-MIT) |
| rust | quick-xml | 0.41.0 | MIT | [LICENSE-MIT.md](licenses/rust/quick-xml-0.41.0/LICENSE-MIT.md) |
| rust | quinn-proto | 0.11.16 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/quinn-proto-0.11.16/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/quinn-proto-0.11.16/LICENSE-MIT) |
| rust | quinn-udp | 0.5.15 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/quinn-udp-0.5.15/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/quinn-udp-0.5.15/LICENSE-MIT) |
| rust | quinn | 0.11.11 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/quinn-0.11.11/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/quinn-0.11.11/LICENSE-MIT) |
| rust | quote | 1.0.47 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/quote-1.0.47/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/quote-1.0.47/LICENSE-MIT) |
| rust | rand | 0.10.2 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand-0.10.2/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand-0.10.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand-0.10.2/LICENSE-MIT) |
| rust | rand | 0.9.5 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand-0.9.5/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand-0.9.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand-0.9.5/LICENSE-MIT) |
| rust | rand_chacha | 0.9.0 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand_chacha-0.9.0/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand_chacha-0.9.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand_chacha-0.9.0/LICENSE-MIT) |
| rust | rand_core | 0.10.1 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand_core-0.10.1/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand_core-0.10.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand_core-0.10.1/LICENSE-MIT) |
| rust | rand_core | 0.6.4 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand_core-0.6.4/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand_core-0.6.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand_core-0.6.4/LICENSE-MIT) |
| rust | rand_core | 0.9.5 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand_core-0.9.5/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand_core-0.9.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand_core-0.9.5/LICENSE-MIT) |
| rust | rand_pcg | 0.10.2 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/rand_pcg-0.10.2/COPYRIGHT), [LICENSE-APACHE](licenses/rust/rand_pcg-0.10.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rand_pcg-0.10.2/LICENSE-MIT) |
| rust | raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | [LICENSE-APACHE.md](licenses/rust/raw-window-handle-0.6.2/LICENSE-APACHE.md), [LICENSE-MIT.md](licenses/rust/raw-window-handle-0.6.2/LICENSE-MIT.md), [LICENSE-ZLIB.md](licenses/rust/raw-window-handle-0.6.2/LICENSE-ZLIB.md) |
| rust | ref-cast-impl | 1.0.26 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ref-cast-impl-1.0.26/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ref-cast-impl-1.0.26/LICENSE-MIT) |
| rust | ref-cast | 1.0.26 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ref-cast-1.0.26/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ref-cast-1.0.26/LICENSE-MIT) |
| rust | regex-automata | 0.4.18 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/regex-automata-0.4.18/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/regex-automata-0.4.18/LICENSE-MIT) |
| rust | regex-syntax | 0.8.11 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/regex-syntax-0.8.11/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/regex-syntax-0.8.11/LICENSE-MIT) |
| rust | regex | 1.13.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/regex-1.13.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/regex-1.13.1/LICENSE-MIT) |
| rust | reqwest | 0.12.28 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/reqwest-0.12.28/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/reqwest-0.12.28/LICENSE-MIT) |
| rust | reqwest | 0.13.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/reqwest-0.13.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/reqwest-0.13.4/LICENSE-MIT) |
| rust | rfd | 0.16.0 | MIT | [LICENSE](licenses/rust/rfd-0.16.0/LICENSE) |
| rust | ring | 0.17.14 | Apache-2.0 AND ISC | [LICENSE](licenses/rust/ring-0.17.14/LICENSE), [LICENSE-BoringSSL](licenses/rust/ring-0.17.14/LICENSE-BoringSSL), [LICENSE-other-bits](licenses/rust/ring-0.17.14/LICENSE-other-bits) |
| rust | rusqlite | 0.32.1 | MIT | [LICENSE](licenses/rust/rusqlite-0.32.1/LICENSE) |
| rust | rustc-hash | 2.1.3 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/rustc-hash-2.1.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rustc-hash-2.1.3/LICENSE-MIT) |
| rust | rustc_version | 0.4.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/rustc_version-0.4.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rustc_version-0.4.1/LICENSE-MIT) |
| rust | rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/rustls-pki-types-1.15.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rustls-pki-types-1.15.1/LICENSE-MIT) |
| rust | rustls-platform-verifier | 0.7.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/rustls-platform-verifier-0.7.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/rustls-platform-verifier-0.7.0/LICENSE-MIT) |
| rust | rustls-webpki | 0.103.14 | ISC | [LICENSE](licenses/rust/rustls-webpki-0.103.14/LICENSE) |
| rust | rustls | 0.23.43 | Apache-2.0 OR ISC OR MIT | [LICENSE-APACHE](licenses/rust/rustls-0.23.43/LICENSE-APACHE), [LICENSE-ISC](licenses/rust/rustls-0.23.43/LICENSE-ISC), [LICENSE-MIT](licenses/rust/rustls-0.23.43/LICENSE-MIT) |
| rust | ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | [LICENSE-APACHE](licenses/rust/ryu-1.0.23/LICENSE-APACHE), [LICENSE-BOOST](licenses/rust/ryu-1.0.23/LICENSE-BOOST) |
| rust | same-file | 1.0.6 | Unlicense/MIT | [COPYING](licenses/rust/same-file-1.0.6/COPYING), [LICENSE-MIT](licenses/rust/same-file-1.0.6/LICENSE-MIT) |
| rust | schemars | 0.8.22 | MIT | [LICENSE](licenses/rust/schemars-0.8.22/LICENSE) |
| rust | schemars | 0.9.0 | MIT | [LICENSE](licenses/rust/schemars-0.9.0/LICENSE) |
| rust | schemars | 1.2.2 | MIT | [LICENSE](licenses/rust/schemars-1.2.2/LICENSE) |
| rust | schemars_derive | 0.8.22 | MIT | [LICENSE](licenses/rust/schemars_derive-0.8.22/LICENSE) |
| rust | scopeguard | 1.2.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/scopeguard-1.2.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/scopeguard-1.2.0/LICENSE-MIT) |
| rust | selectors | 0.36.1 | MPL-2.0 | [LICENSE-upstream-MPL-2.0](licenses/rust/selectors-0.36.1/LICENSE-upstream-MPL-2.0) |
| rust | semver | 1.0.28 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/semver-1.0.28/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/semver-1.0.28/LICENSE-MIT) |
| rust | serde-untagged | 0.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde-untagged-0.1.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde-untagged-0.1.9/LICENSE-MIT) |
| rust | serde | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde-1.0.229/LICENSE-MIT) |
| rust | serde_core | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_core-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_core-1.0.229/LICENSE-MIT) |
| rust | serde_derive | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_derive-1.0.229/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_derive-1.0.229/LICENSE-MIT) |
| rust | serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_derive_internals-0.29.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_derive_internals-0.29.1/LICENSE-MIT) |
| rust | serde_json | 1.0.151 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_json-1.0.151/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_json-1.0.151/LICENSE-MIT) |
| rust | serde_repr | 0.1.21 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_repr-0.1.21/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_repr-0.1.21/LICENSE-MIT) |
| rust | serde_spanned | 1.1.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_spanned-1.1.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_spanned-1.1.1/LICENSE-MIT) |
| rust | serde_urlencoded | 0.7.1 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_urlencoded-0.7.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_urlencoded-0.7.1/LICENSE-MIT) |
| rust | serde_with | 3.22.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_with-3.22.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_with-3.22.0/LICENSE-MIT) |
| rust | serde_with_macros | 3.22.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serde_with_macros-3.22.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serde_with_macros-3.22.0/LICENSE-MIT) |
| rust | serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serialize-to-javascript-impl-0.1.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serialize-to-javascript-impl-0.1.2/LICENSE-MIT) |
| rust | serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/serialize-to-javascript-0.1.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/serialize-to-javascript-0.1.2/LICENSE-MIT) |
| rust | servo_arc | 0.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/servo_arc-0.4.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/servo_arc-0.4.3/LICENSE-MIT) |
| rust | sha1 | 0.10.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/sha1-0.10.7/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/sha1-0.10.7/LICENSE-MIT) |
| rust | sha2 | 0.10.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/sha2-0.10.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/sha2-0.10.9/LICENSE-MIT) |
| rust | shlex | 2.0.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/shlex-2.0.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/shlex-2.0.1/LICENSE-MIT) |
| rust | simd-adler32 | 0.3.10 | MIT | [LICENSE.md](licenses/rust/simd-adler32-0.3.10/LICENSE.md) |
| rust | siphasher | 1.0.3 | MIT/Apache-2.0 | [COPYING](licenses/rust/siphasher-1.0.3/COPYING) |
| rust | slab | 0.4.12 | MIT | [LICENSE](licenses/rust/slab-0.4.12/LICENSE) |
| rust | smallvec | 1.15.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/smallvec-1.15.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/smallvec-1.15.2/LICENSE-MIT) |
| rust | socket2 | 0.6.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/socket2-0.6.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/socket2-0.6.5/LICENSE-MIT) |
| rust | softbuffer | 0.4.8 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/softbuffer-0.4.8/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/softbuffer-0.4.8/LICENSE-MIT) |
| rust | stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/stable_deref_trait-1.2.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/stable_deref_trait-1.2.1/LICENSE-MIT) |
| rust | string_cache | 0.9.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/string_cache-0.9.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/string_cache-0.9.0/LICENSE-MIT) |
| rust | string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/string_cache_codegen-0.6.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/string_cache_codegen-0.6.1/LICENSE-MIT) |
| rust | strsim | 0.11.1 | MIT | [LICENSE](licenses/rust/strsim-0.11.1/LICENSE) |
| rust | subtle | 2.6.1 | BSD-3-Clause | [LICENSE](licenses/rust/subtle-2.6.1/LICENSE) |
| rust | syn | 2.0.119 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/syn-2.0.119/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/syn-2.0.119/LICENSE-MIT) |
| rust | syn | 3.0.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/syn-3.0.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/syn-3.0.3/LICENSE-MIT) |
| rust | sync_wrapper | 1.0.2 | Apache-2.0 | [LICENSE](licenses/rust/sync_wrapper-1.0.2/LICENSE) |
| rust | synstructure | 0.13.2 | MIT | [LICENSE](licenses/rust/synstructure-0.13.2/LICENSE) |
| rust | tao | 0.35.3 | Apache-2.0 | [LICENSE](licenses/rust/tao-0.35.3/LICENSE), [LICENSE.spdx](licenses/rust/tao-0.35.3/LICENSE.spdx) |
| rust | tauri-build | 2.6.3 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-build-2.6.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-build-2.6.3/LICENSE_MIT) |
| rust | tauri-codegen | 2.6.3 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-codegen-2.6.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-codegen-2.6.3/LICENSE_MIT) |
| rust | tauri-macros | 2.6.3 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-macros-2.6.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-macros-2.6.3/LICENSE_MIT) |
| rust | tauri-plugin-autostart | 2.5.1 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-autostart-2.5.1/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-autostart-2.5.1/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-autostart-2.5.1/LICENSE_MIT) |
| rust | tauri-plugin-dialog | 2.7.2 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-dialog-2.7.2/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-dialog-2.7.2/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-dialog-2.7.2/LICENSE_MIT) |
| rust | tauri-plugin-fs | 2.5.1 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-fs-2.5.1/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-fs-2.5.1/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-fs-2.5.1/LICENSE_MIT) |
| rust | tauri-plugin-notification | 2.3.3 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-notification-2.3.3/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-notification-2.3.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-notification-2.3.3/LICENSE_MIT) |
| rust | tauri-plugin-opener | 2.5.4 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-opener-2.5.4/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-opener-2.5.4/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-opener-2.5.4/LICENSE_MIT) |
| rust | tauri-plugin-single-instance | 2.4.4 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-single-instance-2.4.4/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-single-instance-2.4.4/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-single-instance-2.4.4/LICENSE_MIT) |
| rust | tauri-plugin-updater | 2.10.1 | Apache-2.0 OR MIT | [LICENSE.spdx](licenses/rust/tauri-plugin-updater-2.10.1/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-plugin-updater-2.10.1/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-plugin-updater-2.10.1/LICENSE_MIT) |
| rust | tauri-plugin | 2.6.3 | Apache-2.0 OR MIT | [LICENSE-upstream-LICENSE_MIT](licenses/rust/tauri-plugin-2.6.3/LICENSE-upstream-LICENSE_MIT) |
| rust | tauri-runtime-wry | 2.11.4 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-runtime-wry-2.11.4/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-runtime-wry-2.11.4/LICENSE_MIT) |
| rust | tauri-runtime | 2.11.3 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-runtime-2.11.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-runtime-2.11.3/LICENSE_MIT) |
| rust | tauri-utils | 2.9.3 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-utils-2.9.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-utils-2.9.3/LICENSE_MIT) |
| rust | tauri-winres | 0.3.6 | MIT | [LICENSE](licenses/rust/tauri-winres-0.3.6/LICENSE) |
| rust | tauri-winrt-notification | 0.7.3 | MIT OR Apache-2.0 | [LICENSE.spdx](licenses/rust/tauri-winrt-notification-0.7.3/LICENSE.spdx), [LICENSE_APACHE-2.0](licenses/rust/tauri-winrt-notification-0.7.3/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-winrt-notification-0.7.3/LICENSE_MIT) |
| rust | tauri | 2.11.5 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/rust/tauri-2.11.5/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/rust/tauri-2.11.5/LICENSE_MIT) |
| rust | tempfile | 3.27.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tempfile-3.27.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tempfile-3.27.0/LICENSE-MIT) |
| rust | tendril | 0.5.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tendril-0.5.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tendril-0.5.1/LICENSE-MIT) |
| rust | thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/thiserror-impl-1.0.69/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/thiserror-impl-1.0.69/LICENSE-MIT) |
| rust | thiserror-impl | 2.0.20 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/thiserror-impl-2.0.20/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/thiserror-impl-2.0.20/LICENSE-MIT) |
| rust | thiserror | 1.0.69 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/thiserror-1.0.69/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/thiserror-1.0.69/LICENSE-MIT) |
| rust | thiserror | 2.0.20 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/thiserror-2.0.20/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/thiserror-2.0.20/LICENSE-MIT) |
| rust | time-core | 0.1.9 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/time-core-0.1.9/LICENSE-Apache), [LICENSE-MIT](licenses/rust/time-core-0.1.9/LICENSE-MIT) |
| rust | time-macros | 0.2.32 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/time-macros-0.2.32/LICENSE-Apache), [LICENSE-MIT](licenses/rust/time-macros-0.2.32/LICENSE-MIT) |
| rust | time | 0.3.55 | MIT OR Apache-2.0 | [LICENSE-Apache](licenses/rust/time-0.3.55/LICENSE-Apache), [LICENSE-MIT](licenses/rust/time-0.3.55/LICENSE-MIT) |
| rust | tiny_http | 0.12.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tiny_http-0.12.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tiny_http-0.12.0/LICENSE-MIT) |
| rust | tinystr | 0.8.4 | Unicode-3.0 | [LICENSE](licenses/rust/tinystr-0.8.4/LICENSE) |
| rust | tinyvec | 1.12.0 | Zlib OR Apache-2.0 OR MIT | [LICENSE-APACHE.md](licenses/rust/tinyvec-1.12.0/LICENSE-APACHE.md), [LICENSE-MIT.md](licenses/rust/tinyvec-1.12.0/LICENSE-MIT.md), [LICENSE-ZLIB.md](licenses/rust/tinyvec-1.12.0/LICENSE-ZLIB.md) |
| rust | tinyvec_macros | 0.1.1 | MIT OR Apache-2.0 OR Zlib | [LICENSE-APACHE.md](licenses/rust/tinyvec_macros-0.1.1/LICENSE-APACHE.md), [LICENSE-MIT.md](licenses/rust/tinyvec_macros-0.1.1/LICENSE-MIT.md), [LICENSE-ZLIB.md](licenses/rust/tinyvec_macros-0.1.1/LICENSE-ZLIB.md) |
| rust | tokio-macros | 2.7.2 | MIT | [LICENSE](licenses/rust/tokio-macros-2.7.2/LICENSE) |
| rust | tokio-rustls | 0.26.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tokio-rustls-0.26.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tokio-rustls-0.26.4/LICENSE-MIT) |
| rust | tokio-util | 0.7.19 | MIT | [LICENSE](licenses/rust/tokio-util-0.7.19/LICENSE) |
| rust | tokio | 1.53.1 | MIT | [LICENSE](licenses/rust/tokio-1.53.1/LICENSE) |
| rust | toml | 0.9.12+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml-0.9.12_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml-0.9.12_spec-1.1.0/LICENSE-MIT) |
| rust | toml | 1.1.4+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml-1.1.4_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml-1.1.4_spec-1.1.0/LICENSE-MIT) |
| rust | toml_datetime | 0.7.5+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml_datetime-0.7.5_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml_datetime-0.7.5_spec-1.1.0/LICENSE-MIT) |
| rust | toml_datetime | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml_datetime-1.1.1_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml_datetime-1.1.1_spec-1.1.0/LICENSE-MIT) |
| rust | toml_parser | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml_parser-1.1.3_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml_parser-1.1.3_spec-1.1.0/LICENSE-MIT) |
| rust | toml_writer | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/toml_writer-1.1.2_spec-1.1.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/toml_writer-1.1.2_spec-1.1.0/LICENSE-MIT) |
| rust | tower-http | 0.6.11 | MIT | [LICENSE](licenses/rust/tower-http-0.6.11/LICENSE) |
| rust | tower-layer | 0.3.3 | MIT | [LICENSE](licenses/rust/tower-layer-0.3.3/LICENSE) |
| rust | tower-service | 0.3.3 | MIT | [LICENSE](licenses/rust/tower-service-0.3.3/LICENSE) |
| rust | tower | 0.5.3 | MIT | [LICENSE](licenses/rust/tower-0.5.3/LICENSE) |
| rust | tracing-attributes | 0.1.31 | MIT | [LICENSE](licenses/rust/tracing-attributes-0.1.31/LICENSE) |
| rust | tracing-core | 0.1.36 | MIT | [LICENSE](licenses/rust/tracing-core-0.1.36/LICENSE) |
| rust | tracing | 0.1.44 | MIT | [LICENSE](licenses/rust/tracing-0.1.44/LICENSE) |
| rust | tray-icon | 0.24.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tray-icon-0.24.2/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tray-icon-0.24.2/LICENSE-MIT), [LICENSE.spdx](licenses/rust/tray-icon-0.24.2/LICENSE.spdx) |
| rust | try-lock | 0.2.5 | MIT | [LICENSE](licenses/rust/try-lock-0.2.5/LICENSE) |
| rust | tungstenite | 0.28.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/tungstenite-0.28.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/tungstenite-0.28.0/LICENSE-MIT) |
| rust | typeid | 1.0.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/typeid-1.0.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/typeid-1.0.3/LICENSE-MIT) |
| rust | typenum | 1.20.1 | MIT OR Apache-2.0 | [LICENSE](licenses/rust/typenum-1.20.1/LICENSE), [LICENSE-APACHE](licenses/rust/typenum-1.20.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/typenum-1.20.1/LICENSE-MIT) |
| rust | unic-char-property | 0.9.0 | MIT/Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/unic-char-property-0.9.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/unic-char-property-0.9.0/LICENSE-upstream-LICENSE-MIT) |
| rust | unic-char-range | 0.9.0 | MIT/Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/unic-char-range-0.9.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/unic-char-range-0.9.0/LICENSE-upstream-LICENSE-MIT) |
| rust | unic-common | 0.9.0 | MIT/Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/unic-common-0.9.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/unic-common-0.9.0/LICENSE-upstream-LICENSE-MIT) |
| rust | unic-ucd-ident | 0.9.0 | MIT/Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/unic-ucd-ident-0.9.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/unic-ucd-ident-0.9.0/LICENSE-upstream-LICENSE-MIT) |
| rust | unic-ucd-version | 0.9.0 | MIT/Apache-2.0 | [LICENSE-upstream-LICENSE-APACHE](licenses/rust/unic-ucd-version-0.9.0/LICENSE-upstream-LICENSE-APACHE), [LICENSE-upstream-LICENSE-MIT](licenses/rust/unic-ucd-version-0.9.0/LICENSE-upstream-LICENSE-MIT) |
| rust | unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | [LICENSE-APACHE](licenses/rust/unicode-ident-1.0.24/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/unicode-ident-1.0.24/LICENSE-MIT), [LICENSE-UNICODE](licenses/rust/unicode-ident-1.0.24/LICENSE-UNICODE) |
| rust | unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | [COPYRIGHT](licenses/rust/unicode-segmentation-1.13.3/COPYRIGHT), [LICENSE-APACHE](licenses/rust/unicode-segmentation-1.13.3/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/unicode-segmentation-1.13.3/LICENSE-MIT) |
| rust | universal-hash | 0.5.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/universal-hash-0.5.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/universal-hash-0.5.1/LICENSE-MIT) |
| rust | untrusted | 0.9.0 | ISC | [LICENSE.txt](licenses/rust/untrusted-0.9.0/LICENSE.txt) |
| rust | ureq | 2.12.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/ureq-2.12.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/ureq-2.12.1/LICENSE-MIT) |
| rust | url | 2.5.8 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/url-2.5.8/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/url-2.5.8/LICENSE-MIT) |
| rust | urlpattern | 0.3.0 | MIT | [LICENSE](licenses/rust/urlpattern-0.3.0/LICENSE) |
| rust | utf-8 | 0.7.6 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/utf-8-0.7.6/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/utf-8-0.7.6/LICENSE-MIT) |
| rust | utf8_iter | 1.0.4 | Apache-2.0 OR MIT | [COPYRIGHT](licenses/rust/utf8_iter-1.0.4/COPYRIGHT), [LICENSE-APACHE](licenses/rust/utf8_iter-1.0.4/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/utf8_iter-1.0.4/LICENSE-MIT) |
| rust | uuid | 1.24.1 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/uuid-1.24.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/uuid-1.24.1/LICENSE-MIT) |
| rust | vcpkg | 0.2.15 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/vcpkg-0.2.15/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/vcpkg-0.2.15/LICENSE-MIT) |
| rust | version_check | 0.9.5 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/version_check-0.9.5/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/version_check-0.9.5/LICENSE-MIT) |
| rust | vswhom-sys | 0.1.3 | MIT | [LICENSE](licenses/rust/vswhom-sys-0.1.3/LICENSE) |
| rust | vswhom | 0.1.0 | MIT | [LICENSE](licenses/rust/vswhom-0.1.0/LICENSE) |
| rust | walkdir | 2.5.0 | Unlicense/MIT | [COPYING](licenses/rust/walkdir-2.5.0/COPYING), [LICENSE-MIT](licenses/rust/walkdir-2.5.0/LICENSE-MIT) |
| rust | want | 0.3.1 | MIT | [LICENSE](licenses/rust/want-0.3.1/LICENSE) |
| rust | web_atoms | 0.2.6 | MIT OR Apache-2.0 | [LICENSE-APACHE](licenses/rust/web_atoms-0.2.6/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/web_atoms-0.2.6/LICENSE-MIT) |
| rust | webpki-roots | 1.0.9 | CDLA-Permissive-2.0 | [LICENSE](licenses/rust/webpki-roots-1.0.9/LICENSE) |
| rust | webview2-com-macros | 0.8.1 | MIT | [LICENSE-upstream-LICENSE](licenses/rust/webview2-com-macros-0.8.1/LICENSE-upstream-LICENSE) |
| rust | webview2-com-sys | 0.38.2 | MIT | [LICENSE-upstream-LICENSE](licenses/rust/webview2-com-sys-0.38.2/LICENSE-upstream-LICENSE) |
| rust | webview2-com | 0.38.2 | MIT | [LICENSE-upstream-LICENSE](licenses/rust/webview2-com-0.38.2/LICENSE-upstream-LICENSE) |
| rust | winapi-util | 0.1.11 | Unlicense OR MIT | [COPYING](licenses/rust/winapi-util-0.1.11/COPYING), [LICENSE-MIT](licenses/rust/winapi-util-0.1.11/LICENSE-MIT) |
| rust | winapi | 0.3.9 | MIT/Apache-2.0 | [LICENSE-APACHE](licenses/rust/winapi-0.3.9/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/winapi-0.3.9/LICENSE-MIT) |
| rust | window-vibrancy | 0.6.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/window-vibrancy-0.6.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/window-vibrancy-0.6.0/LICENSE-MIT), [LICENSE.spdx](licenses/rust/window-vibrancy-0.6.0/LICENSE.spdx) |
| rust | windows-collections | 0.2.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-collections-0.2.0/license-apache-2.0), [license-mit](licenses/rust/windows-collections-0.2.0/license-mit) |
| rust | windows-core | 0.58.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-core-0.58.0/license-apache-2.0), [license-mit](licenses/rust/windows-core-0.58.0/license-mit) |
| rust | windows-core | 0.61.2 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-core-0.61.2/license-apache-2.0), [license-mit](licenses/rust/windows-core-0.61.2/license-mit) |
| rust | windows-future | 0.2.1 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-future-0.2.1/license-apache-2.0), [license-mit](licenses/rust/windows-future-0.2.1/license-mit) |
| rust | windows-implement | 0.58.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-implement-0.58.0/license-apache-2.0), [license-mit](licenses/rust/windows-implement-0.58.0/license-mit) |
| rust | windows-implement | 0.60.2 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-implement-0.60.2/license-apache-2.0), [license-mit](licenses/rust/windows-implement-0.60.2/license-mit) |
| rust | windows-interface | 0.58.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-interface-0.58.0/license-apache-2.0), [license-mit](licenses/rust/windows-interface-0.58.0/license-mit) |
| rust | windows-interface | 0.59.3 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-interface-0.59.3/license-apache-2.0), [license-mit](licenses/rust/windows-interface-0.59.3/license-mit) |
| rust | windows-link | 0.1.3 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-link-0.1.3/license-apache-2.0), [license-mit](licenses/rust/windows-link-0.1.3/license-mit) |
| rust | windows-link | 0.2.1 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-link-0.2.1/license-apache-2.0), [license-mit](licenses/rust/windows-link-0.2.1/license-mit) |
| rust | windows-numerics | 0.2.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-numerics-0.2.0/license-apache-2.0), [license-mit](licenses/rust/windows-numerics-0.2.0/license-mit) |
| rust | windows-result | 0.2.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-result-0.2.0/license-apache-2.0), [license-mit](licenses/rust/windows-result-0.2.0/license-mit) |
| rust | windows-result | 0.3.4 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-result-0.3.4/license-apache-2.0), [license-mit](licenses/rust/windows-result-0.3.4/license-mit) |
| rust | windows-strings | 0.1.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-strings-0.1.0/license-apache-2.0), [license-mit](licenses/rust/windows-strings-0.1.0/license-mit) |
| rust | windows-strings | 0.4.2 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-strings-0.4.2/license-apache-2.0), [license-mit](licenses/rust/windows-strings-0.4.2/license-mit) |
| rust | windows-sys | 0.48.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-sys-0.48.0/license-apache-2.0), [license-mit](licenses/rust/windows-sys-0.48.0/license-mit) |
| rust | windows-sys | 0.52.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-sys-0.52.0/license-apache-2.0), [license-mit](licenses/rust/windows-sys-0.52.0/license-mit) |
| rust | windows-sys | 0.59.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-sys-0.59.0/license-apache-2.0), [license-mit](licenses/rust/windows-sys-0.59.0/license-mit) |
| rust | windows-sys | 0.60.2 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-sys-0.60.2/license-apache-2.0), [license-mit](licenses/rust/windows-sys-0.60.2/license-mit) |
| rust | windows-sys | 0.61.2 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-sys-0.61.2/license-apache-2.0), [license-mit](licenses/rust/windows-sys-0.61.2/license-mit) |
| rust | windows-targets | 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-targets-0.48.5/license-apache-2.0), [license-mit](licenses/rust/windows-targets-0.48.5/license-mit) |
| rust | windows-targets | 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-targets-0.52.6/license-apache-2.0), [license-mit](licenses/rust/windows-targets-0.52.6/license-mit) |
| rust | windows-targets | 0.53.5 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-targets-0.53.5/license-apache-2.0), [license-mit](licenses/rust/windows-targets-0.53.5/license-mit) |
| rust | windows-threading | 0.1.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-threading-0.1.0/license-apache-2.0), [license-mit](licenses/rust/windows-threading-0.1.0/license-mit) |
| rust | windows-version | 0.1.7 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-version-0.1.7/license-apache-2.0), [license-mit](licenses/rust/windows-version-0.1.7/license-mit) |
| rust | windows | 0.58.0 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-0.58.0/license-apache-2.0), [license-mit](licenses/rust/windows-0.58.0/license-mit) |
| rust | windows | 0.61.3 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows-0.61.3/license-apache-2.0), [license-mit](licenses/rust/windows-0.61.3/license-mit) |
| rust | windows_x86_64_msvc | 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows_x86_64_msvc-0.48.5/license-apache-2.0), [license-mit](licenses/rust/windows_x86_64_msvc-0.48.5/license-mit) |
| rust | windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows_x86_64_msvc-0.52.6/license-apache-2.0), [license-mit](licenses/rust/windows_x86_64_msvc-0.52.6/license-mit) |
| rust | windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](licenses/rust/windows_x86_64_msvc-0.53.1/license-apache-2.0), [license-mit](licenses/rust/windows_x86_64_msvc-0.53.1/license-mit) |
| rust | winnow | 0.7.15 | MIT | [LICENSE-MIT](licenses/rust/winnow-0.7.15/LICENSE-MIT) |
| rust | winnow | 1.0.4 | MIT | [LICENSE-MIT](licenses/rust/winnow-1.0.4/LICENSE-MIT) |
| rust | winreg | 0.10.1 | MIT | [LICENSE](licenses/rust/winreg-0.10.1/LICENSE) |
| rust | winreg | 0.55.0 | MIT | [LICENSE](licenses/rust/winreg-0.55.0/LICENSE) |
| rust | writeable | 0.6.4 | Unicode-3.0 | [LICENSE](licenses/rust/writeable-0.6.4/LICENSE) |
| rust | wry | 0.55.1 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/wry-0.55.1/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/wry-0.55.1/LICENSE-MIT), [LICENSE.spdx](licenses/rust/wry-0.55.1/LICENSE.spdx) |
| rust | yoke-derive | 0.8.2 | Unicode-3.0 | [LICENSE](licenses/rust/yoke-derive-0.8.2/LICENSE) |
| rust | yoke | 0.8.3 | Unicode-3.0 | [LICENSE](licenses/rust/yoke-0.8.3/LICENSE) |
| rust | zerocopy | 0.8.56 | BSD-2-Clause OR Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/zerocopy-0.8.56/LICENSE-APACHE), [LICENSE-BSD](licenses/rust/zerocopy-0.8.56/LICENSE-BSD), [LICENSE-MIT](licenses/rust/zerocopy-0.8.56/LICENSE-MIT) |
| rust | zerofrom-derive | 0.1.7 | Unicode-3.0 | [LICENSE](licenses/rust/zerofrom-derive-0.1.7/LICENSE) |
| rust | zerofrom | 0.1.8 | Unicode-3.0 | [LICENSE](licenses/rust/zerofrom-0.1.8/LICENSE) |
| rust | zeroize | 1.9.0 | Apache-2.0 OR MIT | [LICENSE-APACHE](licenses/rust/zeroize-1.9.0/LICENSE-APACHE), [LICENSE-MIT](licenses/rust/zeroize-1.9.0/LICENSE-MIT) |
| rust | zerotrie | 0.2.5 | Unicode-3.0 | [LICENSE](licenses/rust/zerotrie-0.2.5/LICENSE) |
| rust | zerovec-derive | 0.11.4 | Unicode-3.0 | [LICENSE](licenses/rust/zerovec-derive-0.11.4/LICENSE) |
| rust | zerovec | 0.11.7 | Unicode-3.0 | [LICENSE](licenses/rust/zerovec-0.11.7/LICENSE) |
| rust | zip | 4.6.1 | MIT | [LICENSE](licenses/rust/zip-4.6.1/LICENSE) |
| rust | zmij | 1.0.23 | MIT | [LICENSE-MIT](licenses/rust/zmij-1.0.23/LICENSE-MIT) |
| npm | @babel/runtime | 7.29.7 | MIT | [LICENSE](licenses/npm/_babel_runtime-7.29.7/LICENSE) |
| npm | @floating-ui/core | 1.8.0 | MIT | [LICENSE](licenses/npm/_floating-ui_core-1.8.0/LICENSE) |
| npm | @floating-ui/dom | 1.8.0 | MIT | [LICENSE](licenses/npm/_floating-ui_dom-1.8.0/LICENSE) |
| npm | @floating-ui/react-dom | 2.1.9 | MIT | [LICENSE](licenses/npm/_floating-ui_react-dom-2.1.9/LICENSE) |
| npm | @floating-ui/utils | 0.2.12 | MIT | [LICENSE](licenses/npm/_floating-ui_utils-0.2.12/LICENSE) |
| npm | @fontsource-variable/bricolage-grotesque | 5.3.0 | OFL-1.1 | [LICENSE](licenses/npm/_fontsource-variable_bricolage-grotesque-5.3.0/LICENSE) |
| npm | @radix-ui/number | 1.1.3 | MIT | [LICENSE](licenses/npm/_radix-ui_number-1.1.3/LICENSE) |
| npm | @radix-ui/primitive | 1.1.7 | MIT | [LICENSE](licenses/npm/_radix-ui_primitive-1.1.7/LICENSE) |
| npm | @radix-ui/react-accordion | 1.2.20 | MIT | [LICENSE](licenses/npm/_radix-ui_react-accordion-1.2.20/LICENSE) |
| npm | @radix-ui/react-alert-dialog | 1.1.23 | MIT | [LICENSE](licenses/npm/_radix-ui_react-alert-dialog-1.1.23/LICENSE) |
| npm | @radix-ui/react-arrow | 1.1.15 | MIT | [LICENSE](licenses/npm/_radix-ui_react-arrow-1.1.15/LICENSE) |
| npm | @radix-ui/react-checkbox | 1.3.11 | MIT | [LICENSE](licenses/npm/_radix-ui_react-checkbox-1.3.11/LICENSE) |
| npm | @radix-ui/react-collapsible | 1.1.20 | MIT | [LICENSE](licenses/npm/_radix-ui_react-collapsible-1.1.20/LICENSE) |
| npm | @radix-ui/react-collection | 1.1.15 | MIT | [LICENSE](licenses/npm/_radix-ui_react-collection-1.1.15/LICENSE) |
| npm | @radix-ui/react-compose-refs | 1.1.5 | MIT | [LICENSE](licenses/npm/_radix-ui_react-compose-refs-1.1.5/LICENSE) |
| npm | @radix-ui/react-context | 1.2.2 | MIT | [LICENSE](licenses/npm/_radix-ui_react-context-1.2.2/LICENSE) |
| npm | @radix-ui/react-dialog | 1.1.23 | MIT | [LICENSE](licenses/npm/_radix-ui_react-dialog-1.1.23/LICENSE) |
| npm | @radix-ui/react-direction | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-direction-1.1.4/LICENSE) |
| npm | @radix-ui/react-dismissable-layer | 1.1.19 | MIT | [LICENSE](licenses/npm/_radix-ui_react-dismissable-layer-1.1.19/LICENSE) |
| npm | @radix-ui/react-dropdown-menu | 2.1.24 | MIT | [LICENSE](licenses/npm/_radix-ui_react-dropdown-menu-2.1.24/LICENSE) |
| npm | @radix-ui/react-focus-guards | 1.1.6 | MIT | [LICENSE](licenses/npm/_radix-ui_react-focus-guards-1.1.6/LICENSE) |
| npm | @radix-ui/react-focus-scope | 1.1.16 | MIT | [LICENSE](licenses/npm/_radix-ui_react-focus-scope-1.1.16/LICENSE) |
| npm | @radix-ui/react-hover-card | 1.1.23 | MIT | [LICENSE](licenses/npm/_radix-ui_react-hover-card-1.1.23/LICENSE) |
| npm | @radix-ui/react-id | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-id-1.1.4/LICENSE) |
| npm | @radix-ui/react-label | 2.1.15 | MIT | [LICENSE](licenses/npm/_radix-ui_react-label-2.1.15/LICENSE) |
| npm | @radix-ui/react-menu | 2.1.24 | MIT | [LICENSE](licenses/npm/_radix-ui_react-menu-2.1.24/LICENSE) |
| npm | @radix-ui/react-popover | 1.1.23 | MIT | [LICENSE](licenses/npm/_radix-ui_react-popover-1.1.23/LICENSE) |
| npm | @radix-ui/react-popper | 1.3.7 | MIT | [LICENSE](licenses/npm/_radix-ui_react-popper-1.3.7/LICENSE) |
| npm | @radix-ui/react-portal | 1.1.17 | MIT | [LICENSE](licenses/npm/_radix-ui_react-portal-1.1.17/LICENSE) |
| npm | @radix-ui/react-presence | 1.1.10 | MIT | [LICENSE](licenses/npm/_radix-ui_react-presence-1.1.10/LICENSE) |
| npm | @radix-ui/react-primitive | 2.1.10 | MIT | [LICENSE](licenses/npm/_radix-ui_react-primitive-2.1.10/LICENSE) |
| npm | @radix-ui/react-radio-group | 1.4.7 | MIT | [LICENSE](licenses/npm/_radix-ui_react-radio-group-1.4.7/LICENSE) |
| npm | @radix-ui/react-roving-focus | 1.1.19 | MIT | [LICENSE](licenses/npm/_radix-ui_react-roving-focus-1.1.19/LICENSE) |
| npm | @radix-ui/react-scroll-area | 1.2.18 | MIT | [LICENSE](licenses/npm/_radix-ui_react-scroll-area-1.2.18/LICENSE) |
| npm | @radix-ui/react-select | 2.3.7 | MIT | [LICENSE](licenses/npm/_radix-ui_react-select-2.3.7/LICENSE) |
| npm | @radix-ui/react-separator | 1.1.15 | MIT | [LICENSE](licenses/npm/_radix-ui_react-separator-1.1.15/LICENSE) |
| npm | @radix-ui/react-slot | 1.3.3 | MIT | [LICENSE](licenses/npm/_radix-ui_react-slot-1.3.3/LICENSE) |
| npm | @radix-ui/react-switch | 1.3.7 | MIT | [LICENSE](licenses/npm/_radix-ui_react-switch-1.3.7/LICENSE) |
| npm | @radix-ui/react-tabs | 1.1.21 | MIT | [LICENSE](licenses/npm/_radix-ui_react-tabs-1.1.21/LICENSE) |
| npm | @radix-ui/react-tooltip | 1.2.16 | MIT | [LICENSE](licenses/npm/_radix-ui_react-tooltip-1.2.16/LICENSE) |
| npm | @radix-ui/react-use-callback-ref | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-callback-ref-1.1.4/LICENSE) |
| npm | @radix-ui/react-use-controllable-state | 1.2.6 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-controllable-state-1.2.6/LICENSE) |
| npm | @radix-ui/react-use-effect-event | 0.0.5 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-effect-event-0.0.5/LICENSE) |
| npm | @radix-ui/react-use-is-hydrated | 0.1.3 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-is-hydrated-0.1.3/LICENSE) |
| npm | @radix-ui/react-use-layout-effect | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-layout-effect-1.1.4/LICENSE) |
| npm | @radix-ui/react-use-previous | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-previous-1.1.4/LICENSE) |
| npm | @radix-ui/react-use-rect | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-rect-1.1.4/LICENSE) |
| npm | @radix-ui/react-use-size | 1.1.4 | MIT | [LICENSE](licenses/npm/_radix-ui_react-use-size-1.1.4/LICENSE) |
| npm | @radix-ui/react-visually-hidden | 1.2.11 | MIT | [LICENSE](licenses/npm/_radix-ui_react-visually-hidden-1.2.11/LICENSE) |
| npm | @radix-ui/rect | 1.1.3 | MIT | [LICENSE](licenses/npm/_radix-ui_rect-1.1.3/LICENSE) |
| npm | @tauri-apps/api | 2.11.1 | Apache-2.0 OR MIT | [LICENSE_APACHE-2.0](licenses/npm/_tauri-apps_api-2.11.1/LICENSE_APACHE-2.0), [LICENSE_MIT](licenses/npm/_tauri-apps_api-2.11.1/LICENSE_MIT) |
| npm | @tauri-apps/plugin-dialog | 2.7.2 | MIT OR Apache-2.0 | [LICENSE.spdx](licenses/npm/_tauri-apps_plugin-dialog-2.7.2/LICENSE.spdx) |
| npm | @tauri-apps/plugin-opener | 2.5.4 | MIT OR Apache-2.0 | [LICENSE.spdx](licenses/npm/_tauri-apps_plugin-opener-2.5.4/LICENSE.spdx) |
| npm | @tauri-apps/plugin-updater | 2.10.1 | MIT OR Apache-2.0 | [LICENSE.spdx](licenses/npm/_tauri-apps_plugin-updater-2.10.1/LICENSE.spdx) |
| npm | @types/d3-array | 3.2.2 | MIT | [LICENSE](licenses/npm/_types_d3-array-3.2.2/LICENSE) |
| npm | @types/d3-color | 3.1.3 | MIT | [LICENSE](licenses/npm/_types_d3-color-3.1.3/LICENSE) |
| npm | @types/d3-ease | 3.0.2 | MIT | [LICENSE](licenses/npm/_types_d3-ease-3.0.2/LICENSE) |
| npm | @types/d3-interpolate | 3.0.4 | MIT | [LICENSE](licenses/npm/_types_d3-interpolate-3.0.4/LICENSE) |
| npm | @types/d3-path | 3.1.1 | MIT | [LICENSE](licenses/npm/_types_d3-path-3.1.1/LICENSE) |
| npm | @types/d3-scale | 4.0.9 | MIT | [LICENSE](licenses/npm/_types_d3-scale-4.0.9/LICENSE) |
| npm | @types/d3-shape | 3.2.0 | MIT | [LICENSE](licenses/npm/_types_d3-shape-3.2.0/LICENSE) |
| npm | @types/d3-time | 3.0.4 | MIT | [LICENSE](licenses/npm/_types_d3-time-3.0.4/LICENSE) |
| npm | @types/d3-timer | 3.0.2 | MIT | [LICENSE](licenses/npm/_types_d3-timer-3.0.2/LICENSE) |
| npm | @types/react | 19.2.18 | MIT | [LICENSE](licenses/npm/_types_react-19.2.18/LICENSE) |
| npm | @types/react-dom | 19.2.4 | MIT | [LICENSE](licenses/npm/_types_react-dom-19.2.4/LICENSE) |
| npm | aria-hidden | 1.2.6 | MIT | [LICENSE](licenses/npm/aria-hidden-1.2.6/LICENSE) |
| npm | class-variance-authority | 0.7.1 | Apache-2.0 | [LICENSE](licenses/npm/class-variance-authority-0.7.1/LICENSE) |
| npm | clsx | 2.1.1 | MIT | [license](licenses/npm/clsx-2.1.1/license) |
| npm | cookie | 1.1.1 | MIT | [LICENSE](licenses/npm/cookie-1.1.1/LICENSE) |
| npm | csstype | 3.2.3 | MIT | [LICENSE](licenses/npm/csstype-3.2.3/LICENSE) |
| npm | d3-array | 3.2.4 | ISC | [LICENSE](licenses/npm/d3-array-3.2.4/LICENSE) |
| npm | d3-color | 3.1.0 | ISC | [LICENSE](licenses/npm/d3-color-3.1.0/LICENSE) |
| npm | d3-ease | 3.0.1 | BSD-3-Clause | [LICENSE](licenses/npm/d3-ease-3.0.1/LICENSE) |
| npm | d3-format | 3.1.2 | ISC | [LICENSE](licenses/npm/d3-format-3.1.2/LICENSE) |
| npm | d3-interpolate | 3.0.1 | ISC | [LICENSE](licenses/npm/d3-interpolate-3.0.1/LICENSE) |
| npm | d3-path | 3.1.0 | ISC | [LICENSE](licenses/npm/d3-path-3.1.0/LICENSE) |
| npm | d3-scale | 4.0.2 | ISC | [LICENSE](licenses/npm/d3-scale-4.0.2/LICENSE) |
| npm | d3-shape | 3.2.0 | ISC | [LICENSE](licenses/npm/d3-shape-3.2.0/LICENSE) |
| npm | d3-time | 3.1.0 | ISC | [LICENSE](licenses/npm/d3-time-3.1.0/LICENSE) |
| npm | d3-time-format | 4.1.0 | ISC | [LICENSE](licenses/npm/d3-time-format-4.1.0/LICENSE) |
| npm | d3-timer | 3.0.1 | ISC | [LICENSE](licenses/npm/d3-timer-3.0.1/LICENSE) |
| npm | decimal.js-light | 2.5.1 | MIT | [LICENCE.md](licenses/npm/decimal.js-light-2.5.1/LICENCE.md) |
| npm | detect-node-es | 1.1.0 | MIT | [LICENSE](licenses/npm/detect-node-es-1.1.0/LICENSE) |
| npm | dom-helpers | 5.2.1 | MIT | [LICENSE](licenses/npm/dom-helpers-5.2.1/LICENSE) |
| npm | eventemitter3 | 4.0.7 | MIT | [LICENSE](licenses/npm/eventemitter3-4.0.7/LICENSE) |
| npm | fast-equals | 5.4.1 | MIT | [LICENSE](licenses/npm/fast-equals-5.4.1/LICENSE) |
| npm | get-nonce | 1.0.1 | MIT | [LICENSE](licenses/npm/get-nonce-1.0.1/LICENSE) |
| npm | internmap | 2.0.3 | ISC | [LICENSE](licenses/npm/internmap-2.0.3/LICENSE) |
| npm | js-tokens | 4.0.0 | MIT | [LICENSE](licenses/npm/js-tokens-4.0.0/LICENSE) |
| npm | lodash | 4.18.1 | MIT | [LICENSE](licenses/npm/lodash-4.18.1/LICENSE) |
| npm | loose-envify | 1.4.0 | MIT | [LICENSE](licenses/npm/loose-envify-1.4.0/LICENSE) |
| npm | lucide-react | 1.31.0 | ISC | [LICENSE](licenses/npm/lucide-react-1.31.0/LICENSE) |
| npm | object-assign | 4.1.1 | MIT | [license](licenses/npm/object-assign-4.1.1/license) |
| npm | prop-types | 15.8.1 | MIT | [LICENSE](licenses/npm/prop-types-15.8.1/LICENSE) |
| npm | react-is | 16.13.1 | MIT | [LICENSE](licenses/npm/react-is-16.13.1/LICENSE) |
| npm | react | 19.2.8 | MIT | [LICENSE](licenses/npm/react-19.2.8/LICENSE) |
| npm | react-dom | 19.2.8 | MIT | [LICENSE](licenses/npm/react-dom-19.2.8/LICENSE) |
| npm | react-is | 18.3.1 | MIT | [LICENSE](licenses/npm/react-is-18.3.1/LICENSE) |
| npm | react-remove-scroll | 2.7.2 | MIT | [LICENSE](licenses/npm/react-remove-scroll-2.7.2/LICENSE) |
| npm | react-remove-scroll-bar | 2.3.8 | MIT | [LICENSE-upstream-reference](licenses/npm/react-remove-scroll-bar-2.3.8/LICENSE-upstream-reference) |
| npm | react-router | 7.18.2 | MIT | [LICENSE.md](licenses/npm/react-router-7.18.2/LICENSE.md) |
| npm | react-router-dom | 7.18.2 | MIT | [LICENSE.md](licenses/npm/react-router-dom-7.18.2/LICENSE.md) |
| npm | react-smooth | 4.0.4 | MIT | [LICENSE](licenses/npm/react-smooth-4.0.4/LICENSE) |
| npm | react-style-singleton | 2.2.3 | MIT | [LICENSE](licenses/npm/react-style-singleton-2.2.3/LICENSE) |
| npm | react-transition-group | 4.4.5 | BSD-3-Clause | [LICENSE](licenses/npm/react-transition-group-4.4.5/LICENSE) |
| npm | recharts | 2.15.4 | MIT | [LICENSE](licenses/npm/recharts-2.15.4/LICENSE) |
| npm | recharts-scale | 0.4.5 | MIT | [LICENSE](licenses/npm/recharts-scale-0.4.5/LICENSE) |
| npm | scheduler | 0.27.0 | MIT | [LICENSE](licenses/npm/scheduler-0.27.0/LICENSE) |
| npm | set-cookie-parser | 2.7.2 | MIT | [LICENSE](licenses/npm/set-cookie-parser-2.7.2/LICENSE) |
| npm | sonner | 2.0.8 | MIT | [LICENSE.md](licenses/npm/sonner-2.0.8/LICENSE.md) |
| npm | tailwind-merge | 3.6.0 | MIT | [LICENSE.md](licenses/npm/tailwind-merge-3.6.0/LICENSE.md) |
| npm | tiny-invariant | 1.3.3 | MIT | [LICENSE](licenses/npm/tiny-invariant-1.3.3/LICENSE) |
| npm | tslib | 2.8.1 | 0BSD | [CopyrightNotice.txt](licenses/npm/tslib-2.8.1/CopyrightNotice.txt), [LICENSE.txt](licenses/npm/tslib-2.8.1/LICENSE.txt) |
| npm | tw-animate-css | 1.4.0 | MIT | [LICENSE](licenses/npm/tw-animate-css-1.4.0/LICENSE) |
| npm | use-callback-ref | 1.3.3 | MIT | [LICENSE](licenses/npm/use-callback-ref-1.3.3/LICENSE) |
| npm | use-sidecar | 1.1.3 | MIT | [LICENSE](licenses/npm/use-sidecar-1.1.3/LICENSE) |
| npm | victory-vendor | 36.9.2 | MIT AND ISC | [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-array/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-color/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-ease/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-format/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-interpolate/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-path/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-scale/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-shape/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-time/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-time-format/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-timer/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/d3-voronoi/LICENSE), [LICENSE](licenses/npm/victory-vendor-36.9.2/lib-vendor/internmap/LICENSE), [LICENSE-upstream-v36.9.2](licenses/npm/victory-vendor-36.9.2/LICENSE-upstream-v36.9.2) |
| npm | zustand | 5.0.15 | MIT | [LICENSE](licenses/npm/zustand-5.0.15/LICENSE) |
