# Economics Simulation

## Running

1. Install *Rust*: https://www.rust-lang.org/tools/install
2. Download this repository
3. Open the **server** directory and run `cargo run`
4. Open https://localhost:8080/director_login/
5. For the player view, open https://localhost:8080/login/
6. For the viewer view, open https://localhost:8080/viewer_login/
```bash
git clone https://github.com/Tweoss/econ.git # or download as a zip
cd econ/server && cargo run
```

## Frameworks

There are two main folders, **client** and **server**, in this repository. The **client** code uses [*Yew*](https://yew.rs), a front-end web framework that compiles Rust to WebAssembly. The **server** code uses [*Actix*](https://actix.rs/), a very fast, actor-based **server** framework. Other crates are listed in the Cargo.toml files.

## File Descriptions

format_expr.sh formats a math expression contained in regex.txt so it can be evaluated in *Rust*, *Desmos*, or *Wolfram Alpha*. intersection.m is an *Octave* script file ([Download Octave](https://www.gnu.org/software/octave/download) and run with `octave-cli intersection.m`) that calculates the equilibrium price, quantity, and required funds. 

## Server Logic

The **server** takes post requests (check the [lib.rs](./**client**/login/src/lib.rs) files under the **client** directory) and returns cookies in valid cases. When a **client** attempts to connect to the **server**, these cookies will be checked, allowing players to refresh or navigate away without losing progress. If the cookies are valid, a *WebSocket* connection is established, starting an *Actix* Actor on the server side. This Participant Actor communicates player actions to the Game Actor, which in turn performs calculations and sends messages to Participant Actors. 
