# zkSync Remix Plugin

[![Pull Requests welcome](https://img.shields.io/badge/PRs-welcome-ff69b4.svg?style=flat-square)](https://github.com/NethermindEth/zksync-remix-plugin/issues)
[![Discord](https://img.shields.io/discord/629004402170134531?label=Discord)](https://discord.com/invite/PaCMRFdvWT)

Welcome to the **zkSync Remix Plugin** repository! 🎉

This tool seamlessly integrates with the Remix IDE, enabling developers to effortlessly deploy and interact with zkSync
contracts.
<details> 
<summary>Table of Contents</summary>

- [zkSync Remix Plugin](#zksync-remix-plugin)
    - [For users](#for-users)
        - [Getting Started](#getting-started)
    - [For Developers](#for-developers)
        - [Installation](#installation)
            - [API](#api)
            - [Plugin](#plugin)
            - [Running the development environment](#running-the-development-environment)
                - [Connecting the plugin](#connecting-the-plugin)
    - [Support and Contributions](#support-and-contributions)

</details>

## For users

If you're looking to utilize the capabilities of zkSync contracts within the Remix IDE, you've come to the right place.
This section provides you with a straightforward guide to get started.

### Getting Started

<!-- TODO: temporary link -->

1. **Installation**: Get excited, folks, no complicated installations needed here! If you're a user, all you have to do
   is head over to the Remix IDE and locate the zkSync Remix Plugin in the plugins section. Want to make it even
   simpler? Click right through to Remix using [this direct link](https://remix.ethereum.org/#activate=zkSync) and
   you're good to go! 🎉

2. **Usage**: Once the plugin is activated, you'll find a user-friendly interface that allows you to deploy and interact
   with zkSync contracts. Follow the on-screen prompts and tooltips for an effortlessly smooth experience!

3. **Feedback**: Your feedback is invaluable to us 🌟! If you encounter any issues or have game-changing suggestions,
   don't hesitate to reach out through
   our [Community Forum](https://github.com/zkSync-Community-Hub/zkync-developers/discussions/new?category=remix-plugin).
   Let's make something awesome together! 🤝

## For Developers

### Installation

#### API

Our API is built with [Rocket](https://rocket.rs/), a web framework for Rust. So, you'll need to get Rust and Cargo on
your machine to get started. 🛠️

The easiest way to install Rust and Cargo is by using [rustup](https://rustup.rs/). It's
the [recommended tool](https://www.rust-lang.org/tools/install) for managing Rust versions and associated tools for your
project.

Then:

```bash
cd api
cd hardhat_env
yarn
cd ..
VITE_URL=http://localhost:3000 SERVICE_VERSION="v0.1.1" cargo run
```

#### Plugin

The plugin it self is a React project, you'll need to install [pnpm](https://pnpm.io/installation#using-npm).

```bash
cd plugin;
pnpm install;
```

#### Running the development environment

Firstly, you need to define ENV variables for the server:

- VITE_URL=http://localhost:3000 - the address of the UI
- SERVICE_VERSION="v0.0.1" - the version of the back-end (should be less than version of UI)

And ENV variables for the plugin:

- API_SERVICE_URL="http://0.0.0.0:8000" - the address of the API
- WALLETCONNECT_PROJECT_ID=<your_walletconnect_project_id> - the project id for WalletConnect
- ZKSYNC_DEVNET_URL="https://zksync-devnet.nethermind.dev" - the address of the StarkNet devnet

You need to be running both the server and the plugin in order to have a working environment.

For your dev environment:

```bash
cd plugin;
pnpm run start;
```

For an optimized build (will not listen to changes):

```
pnpm run deploy;
pnpm run serve;
```

```bash
cd api
cd hardhat_env
yarn
cd ..
VITE_URL=http://localhost:3000 SERVICE_VERSION="v0.1.1" cargo run
```

or alternatively, you can run the server in watch mode (with `cargo watch`):

```bash
cargo install cargo-watch;
cargo watch -x run;
```

##### Connecting the plugin

In [Remix](http://remix-alpha.ethereum.org/), go to the `Plugin Manager` at the bottom of the left panel, and click
on `Connect to a Local Plugin`.

Then, chose a name for the plugin, and in the `URL` field, enter `http://localhost:3000`, the `Type of Connection`
should `iframe` and the `Location in remix` `Side Panel` and click on `Ok`.

You should be all set to see the magic happen! Activate the plugin and it should now be visible and ready to be hacked
with! 🚀

## Support and Contributions

Feel free to contribute! Spotted any [issues](https://github.com/NethermindEth/zksync-remix-plugin/issues)? Head on over
to
our [good first issues](https://github.com/NethermindEth/zksync-remix-plugin/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
or read through our [Contribution Guidelines](/docs/CONTRIBUTING.md) to get started. 📝

Jump into our [Discord channel](https://discord.com/invite/PaCMRFdvWT) and join our
thriving [community](https://community.nethermind.io/)! Connect with other users, share insights, and get all your
questions answered. Our community is always eager to help newcomers! 🤝

We're thrilled for you to experience the zkSync Remix Plugin, and we can't wait to see the inventive ways you'll engage
with zkSync contracts! Happy coding! 💡
