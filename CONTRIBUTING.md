[back to README.md](README.md)


# Contributing

## Overview
This project provides the tools for implementing a Holochain pattern to track content affiliation in
a group context.


### Entity Relationship Diagram

![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=1gOWOThwxkcDEIqEbb9Gnrbeo8jcT8a12)

#### Entry Types

- **Group Entry** - *A group state definition*
- **Contributions Anchor Entry** - *An agent's digital property within a group*
- **Archived Contributions Anchor Entry** - *A snapshot of contributions for a specific agent*

#### Link Types

- **Group**
  - *Agent Entry* —> *Group Entry*
- **Group Auth**
  - *Group Entry* —> *Contributions Entry*
- **Group Auth Archive**
  - *Group Entry* —> *Archived Contributions Entry*
- **Contribution**
  - *Contributions Entry* —> *[target]*
  - *Archived Contributions Entry* —> *[target]*
- **Contribution Update**
  - *Contributions Entry* —> *[target]*
  - *Archived Contributions Entry* —> *[target]*


### Integrity Model

See [INTEGRITY_MODEL.md](INTEGRITY_MODEL.md)


## Development

### Environment

- Enter `nix develop` for other development environment dependencies.


### Building

WASM targets

- `make ./zomes/coop_content.wasm` - Integrity Zome
- `make ./zomes/coop_content_csr.wasm` - Default CSR


### Testing

To run all tests with logging
```
make test-debug
```

- `make test-unit` - **Rust tests only**
- `make test-integration-debug` - **Integration tests only**

> **NOTE:** remove `-debug` to run tests without logging
