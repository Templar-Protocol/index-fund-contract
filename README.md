# index-fund-contract

NEAR smart contract for an index fund. 

## Status
The active development of this contract is currently paused until the Templar Team has more bandwidth. If have questions and you'd like to contribute, please join the public Templar Telegram and send us a message: https://t.me/templardiscussion

- [x] basic functionality for registering an index fund curator and updating the weights
- [x] tracking rebalancing
- [ ] fix serialization for test_basics.rs
- [ ] chain sigs for multichain
- [ ] rebalance automatically via intents

## Architecture
https://github.com/Templar-Protocol/architecture/blob/main/design_diagrams/index_fund.md 

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy <account-id>
```

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
