[![](https://img.shields.io/npm/v/@spartan-hc/coop-content-zomelets/latest?style=flat-square)](http://npmjs.com/package/@spartan-hc/coop-content-zomelets)

# Cooperative Content Zomelets
Zomelet implementations for the Cooperative Content zomes.

[![](https://img.shields.io/github/issues-raw/spartan-holochain-counsel/hc-cooperative-content?style=flat-square)](https://github.com/spartan-holochain-counsel/hc-cooperative-content/issues)
[![](https://img.shields.io/github/issues-closed-raw/spartan-holochain-counsel/hc-cooperative-content?style=flat-square)](https://github.com/spartan-holochain-counsel/hc-cooperative-content/issues?q=is%3Aissue+is%3Aclosed)
[![](https://img.shields.io/github/issues-pr-raw/spartan-holochain-counsel/hc-cooperative-content?style=flat-square)](https://github.com/spartan-holochain-counsel/hc-cooperative-content/pulls)


## Install

```bash
npm i @spartan-hc/coop-content-zomelets
```

## Basic Usage

```js
import { CellZomelets } from '@spartan-hc/zomelets';
import { CoopContentZomelet } from '@spartan-hc/coop-content-zomelets';

const cell_interface = CellZomelets({
    "coop_content_csr": CoopContentZomelet,
    // ...your other zomes
});
// Then use `cell_interface` in your Zomelet compatible client
```

See [@spartan-hc/app-interface-client](https://www.npmjs.com/package/@spartan-hc/app-interface-client) for how to use Zomelets.
