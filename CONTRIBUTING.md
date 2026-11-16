# Overview

* [Reporting bugs](#reporting-bugs)
* [Contributing to the code](#contributing-code)
* [Compiling and running Citybound yourself](#compiling-and-running-citybound-yourself)

# Reporting Bugs

If the user interface breaks, you see a red `USER INTERFACE BROKE 😔` message in the browser with error details. If the simulation itself stops working, it usually displays a `SIMULATION BROKE :(` message in an editor that automatically opens, or in a `cb_last_error.txt` file in your system's temporary directory.

**First look if your issue has already been reported:**
* [search all open and closed issues](https://github.com/citybound/citybound/issues?utf8=✓&q=is%3Aissue)

If not, [create a new issue](https://github.com/aeickhoff/citybound/issues/new).

* Please provide details
    * what platform you're on
    * what you were trying to do or what you expected to happen
    * what actually happened
    * the detailed error information
* Ideally: If the game is still running, take one or several screenshots (camera controls continue to work in many cases)
* Perfect: Provide precise instructions on how to reproduce the problem (if possible)

# Contributing Code

## Compiling and running Citybound yourself

**Please note,** newest commits on master might temporarily be broken or represent work-in-progress state.

Recommended setup:
* Install prerequisites
  * Windows/Mac: download and run the installers of [nodejs](https://nodejs.org/en/) and [git](https://git-scm.com/)
  * Windows: additionally install the Visual Studio 2017 build tools from http://aka.ms/buildtools
  * Ubuntu: `sudo apt install npm nodejs git curl libssl-dev pkg-config`
* **Windows: run all following steps inside the Git Bash that came with Git**
  * if you have followed all necessary steps, but Git Bash complains about not finding a command (such as node or rustup), try restarting Git Bash first and `cd citybound` again
* `git clone https://github.com/citybound/citybound.git`
* `cd citybound`
* `npm run ensure-tooling`
  * Follow instructions
  * install rustup if asked to
    * (Mac/Ubuntu: run `source $HOME/.cargo/env` like it suggests)
    * then rerun `npm run ensure-tooling`
* Run the following two commands in parallel in two separate terminals (in the citybound directory)
   * `npm run watch-browser` to continuously build the browser UI 
     * Might take long the first time - installs and compile dependencies
     * Recompiles automatically after changes, just reload browser to get them
   * `npm start` to build and then run the server
     * Might take long the first time - installs and compile dependencies
     * If you get a bug about the `cb_browser_ui/dist` directory missing, wait for `npm run watch-browser` to complete, then restart `npm start`

* Open the displayed address in your browser

## Build system notes (this fork)

The original build setup stopped working years ago (older than the code itself):
`cargo-web 0.6.24` can no longer be installed, `parcel-bundler` needs a native
`deasync` module that does not build on current Node, and the code did not compile
on the Windows backend at all. This fork keeps the game buildable:

* **Rust is pinned to `nightly-2020-03-10`** by `rust-toolchain.toml`. Several
  dependencies (`compact`, `chunky`, …) use nightly-only features that have since
  been removed from the compiler, so a newer toolchain cannot build them. Don't
  "upgrade" the pin without replacing those crates first.
* **`cargo-web` is pinned to `0.6.26`** (installed by `npm run ensure-tooling`).
  The browser UI is a `stdweb` 0.4 / `cargo-web` app; moving off it means a rewrite
  to `wasm-bindgen`.
* **The JS bundle is built with `esbuild`** (`cb_browser_ui/buildJs.js`) and the
  stylesheet with `lessc`, replacing `parcel-bundler` 1.x.
* `cb_util/src/config_manager/mod.rs` used the macOS-only `notify::FsEventWatcher`
  type; it now uses `notify::RecommendedWatcher`, which is what `notify::watcher()`
  actually returns on every platform.
* Git dependencies (`rust-embed-flag`, `noise`) are pinned to commit SHAs so the
  build does not drift when their branches move.

## Guidelines

* **Make sure to <a href="https://www.clahub.com/agreements/citybound/citybound">sign the Contributor License Agreement</a>.**
* **[Have a look at the documentation](http://citybound.github.io/citybound)**
* **Citybound uses trunk-based development,** meaning a very recent work-in-progress state of the code is always in the master branch. The repository owner usually commits directly to master, or uses short-lived feature branches. Contributors use the common fork/pull-request flow and everyone involved tries to get the changes into master as quickly as possible. The newest commits in master might sometimes be broken and not run.

## Conforming to style

* Run `npm run lint` and fix at least formatting issues that couldn't be fixed automatically. If you have time, adress any best-practises issues it raises.

## Have a question? Want to discuss something?

The best place to do so is the [official r/Citybound subreddit](https://reddit.com/r/Citybound)!
