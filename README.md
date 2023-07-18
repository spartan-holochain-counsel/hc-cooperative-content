[![](https://img.shields.io/crates/v/hc_coop_content_sdk?style=flat-square)](https://crates.io/crates/hc_coop_content_sdk)

# Cooperative Content
A set of Zomes (WASMs used in Holochain DNAs) that provide patterns for collaborative content
management.


[![](https://img.shields.io/github/issues-raw/mjbrisebois/hc-cooperative-content?style=flat-square)](https://github.com/mjbrisebois/hc-cooperative-content/issues)
[![](https://img.shields.io/github/issues-closed-raw/mjbrisebois/hc-cooperative-content?style=flat-square)](https://github.com/mjbrisebois/hc-cooperative-content/issues?q=is%3Aissue+is%3Aclosed)
[![](https://img.shields.io/github/issues-pr-raw/mjbrisebois/hc-cooperative-content?style=flat-square)](https://github.com/mjbrisebois/hc-cooperative-content/pulls)

## Overview
Cooperative content is about grouping a set of agents under a common goal and enabling others to
follow the progress of that group.  This project provides a set of rules (integriy zome) for
organizing contributions in an efficient way and a model for updating the list of authorized
contributors.  It also provides a default way of viewing content; however, grouping content in a
Holochain app is essentially just a suggestion because the end-user can ultimately choose how they
want to read the DHT.


### Usage
Implementing Coop Content in a DNA will require that

- The DNA include the `coop_content.wasm` integrity zome and the `coop_content_csr.wasm` coordinator
- The target's coordinator(s) make calls to the Coop Content coordinator
- The target's integrity zome(s) validate group references for target content entries

#### Add WASMs to your DNA config

```diff
  manifest_version: "1"
  name: your_dna
  integrity:
    origin_time: 2023-01-01T00:00:00.000000Z
    network-seed: ~
    properties: ~
    zomes:
      - name: your_zome
        bundled: your_zome.wasm
+     - name: coop_content
+       bundled: coop_content.wasm
  coordinator:
    zomes:
      - name: your_main_csr
        bundled: your_main_csr.wasm
        dependencies:
          - name: your_zome
+     - name: coop_content_csr
+       bundled: coop_content_csr.wasm
+       dependencies:
+         - name: coop_content
```

Real examples in tests
- [./tests/model_dna/dna.yaml](./tests/model_dna/dna.yaml)
- [./tests/minimal_dna/dna.yaml](./tests/minimal_dna/dna.yaml)

#### Add `hc_coop_content_sdk` to `Cargo.toml`

```diff
  [dependencies]
+ hc_coop_content_sdk = "0.1"
```

Real examples in tests
- [./tests/zomes/basic_usage/Cargo.toml](./tests/zomes/basic_usage/Cargo.toml)
- [./tests/zomes/basic_usage_csr/Cargo.toml](./tests/zomes/basic_usage_csr/Cargo.toml)

#### Implement CRUD

See tests for examples

- [./tests/zomes/basic_usage/src/lib.rs](./tests/zomes/basic_usage/src/lib.rs)
- [./tests/zomes/basic_usage_csr/src/lib.rs](./tests/zomes/basic_usage_csr/src/lib.rs)


### API Reference

See [docs/API.md](docs/API.md)

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
