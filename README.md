POC harness for writing Solidity tests in Rust using [ethers-rs](https://github.com/gakonst/ethers-rs).

# Usage 

### Run forge tests
```bash
forge test
```

### Run rust tests
```
forge build
cargo test
```

# Notes
- The usage of [distributed slices](https://github.com/dtolnay/linkme) to link tests can occasionally cause simple syntax errors in a test to trigger an opaque internal compiler error from rustc. If you get an error message claiming to be a bug in rustc, this is probably not the case. Runnning `cargo check --tests` rather than `cargo test` can help clear things up.
