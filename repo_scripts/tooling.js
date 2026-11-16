const util = require('util');
const { execSync, spawnSync } = require('child_process');

// The project is pinned to a 2020 nightly because several of its dependencies
// (`compact`, `chunky`, ...) use nightly-only features that have since been
// removed from the compiler, and the browser UI is built with `cargo-web`,
// which stopped working with later toolchains.
// `rust-toolchain.toml` in the repository root selects this toolchain for
// everything built inside the repo.
const NIGHTLY_VERSION = "nightly-2020-03-10";
// cargo-web 0.6.24 can no longer be installed from source (its macOS-only
// dependency tree resolves to yanked crates); 0.6.26 is a drop-in replacement.
const CARGO_WEB_VERSION = "0.6.26";
let quiet = process.argv[2] == "-q";

let rustupV;
try {
    rustupV = execSync('rustup --version', { encoding: 'utf8' });
} catch (e) {
    rustupV = undefined
}

if (rustupV && rustupV.startsWith("rustup")) {
    !quiet && console.log("Rustup installed ? (OK)");
} else {
    console.log("Rustup missing! ?? (FAIL)");
    console.log("Please install from https://rustup.rs");
    process.exit(1);
}

function ensureRustNightly(nightly) {
    const installed = execSync('rustup toolchain list', { encoding: 'utf8' });
    const hasToolchain = installed.split(/\r?\n/).some(l => l.trim().startsWith(nightly));

    if (!hasToolchain) {
        console.log("Rust toolchain " + nightly + " missing ??(!)");
        console.log("?? Installing " + nightly);
        spawnSync("rustup", ["toolchain", "install", nightly, "--profile", "minimal"], { stdio: 'inherit' });
        quiet = false;
    }

    // the browser UI compiles to WebAssembly
    spawnSync("rustup", ["target", "add", "wasm32-unknown-unknown", "--toolchain", nightly],
        { stdio: quiet ? 'ignore' : 'inherit' }
    );

    const after = execSync('rustup toolchain list', { encoding: 'utf8' });
    if (after.split(/\r?\n/).some(l => l.trim().startsWith(nightly))) {
        !quiet && console.log("Correct rust nightly set up ? (OK)");
    } else {
        console.log("Failed to install correct toolchain ?? (FAIL)");
        process.exit(1);
    }
}

!quiet && console.log("Checking rust nightly");
ensureRustNightly(NIGHTLY_VERSION);

!quiet && console.log("Checking cargo-web version");

function checkCargoWeb(requiredVersion) {
    try {
        let cargoWebVersion = execSync('cargo-web --version', { encoding: 'utf8' });
        if (cargoWebVersion.includes(requiredVersion)) {
            return true;
        }
        return false;
    } catch (e) {
        console.log("Couldn't run cargo-web", e.message);
        return false;
    }
}

if (checkCargoWeb(CARGO_WEB_VERSION)) {
    !quiet && console.log("Correct cargo-web set up ? (OK)");
} else {
    !quiet && console.log("Correct cargo-web not installed yet ??(!)");
    console.log("?? Installing cargo-web");

    let platform = require("os").platform();

    if (platform == "linux" || platform == "darwin") {
        let url = "https://github.com/koute/cargo-web/releases/download/" + CARGO_WEB_VERSION + "/cargo-web-x86_64-" + (platform == "linux" ? "unknown-linux-gnu.gz" : "apple-darwin.gz");
        console.log("Downloading cargo-web executable from " + url);
        console.log(execSync('curl -L ' + url + ' | gzip -d > cargo-web', { encoding: 'utf8' }));
        console.log("Installing cargo-web executable");
        console.log(execSync('chmod +x cargo-web', { encoding: 'utf8' }));
        console.log(execSync('mkdir -p ~/.cargo/bin', { encoding: 'utf8' }));
        console.log(execSync('mv cargo-web ~/.cargo/bin', { encoding: 'utf8' }));
    } else {
        // build it with the current stable toolchain: the 2020 nightly's cargo
        // can no longer resolve cargo-web's dependency tree
        spawnSync("cargo", ["+stable", "install", "cargo-web", "--force", "--vers", CARGO_WEB_VERSION],
            { stdio: quiet ? 'ignore' : 'inherit' }
        );
    }

    if (checkCargoWeb(CARGO_WEB_VERSION)) {
        !quiet && console.log("Correct cargo-web set up ? (OK)");
    } else {
        console.log("Failed to install cargo-web ?? (FAIL)");
        process.exit(1);
    }
}

!quiet && console.log("?? Ensuring linting tools are installed...");
spawnSync("rustup", ["component", "add", "rustfmt-preview", "--toolchain", NIGHTLY_VERSION],
    { stdio: quiet ? 'ignore' : 'inherit' }
);
spawnSync("rustup", ["component", "add", "clippy-preview", "--toolchain", NIGHTLY_VERSION],
    { stdio: quiet ? 'ignore' : 'inherit' }
);
!quiet && console.log("Linting tools set up ? (OK)");
