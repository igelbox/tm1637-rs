# `TM1637`
> A lightweight platform-agnostic driver to a LED-display powered by the TM1637 chip

[![crates.io](https://img.shields.io/crates/v/tm1637.svg)](https://crates.io/crates/tm1637)
[![Released API docs](https://docs.rs/tm1637/badge.svg)](https://docs.rs/tm1637)

## Features
- Formatting number as HEX-digits
- Controlling each particular segment using a bitmask
- Brightness control
- Low ROM usage, e.g. the working example features using the following config take **488**\* additional bytes:
```toml
[profile.release]
codegen-units = 1
debug = true
lto = true
opt-level = 'z'
```
_\* 580 bytes in case if the real HAL delay is used._


## Working Example
A simple example using a 4-digit LED-display is located [here](examples/main.rs).

BTW, it could be built and flashed into STM32C8T6 (Blue-Pill) using Visual Studio Code.
All required extensions are listed [here](.vscode/extensions.json).

## License
Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
