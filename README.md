# Courier

A Telegram bot that generates barcode and QR code images from text input.

## Behavior

| Input | Output |
|-------|--------|
| All-numeric (e.g. `1234567`) | Code128 barcode. If the sender has a configured ID, their barcode is merged above the input barcode. |
| Everything else (e.g. `hello`, `https://example.com`) | QR code |

The bot only responds to private messages and ignores input shorter than 5 characters.

## Configuration

Create a `config.toml` (default path, overridable via CLI argument):

```toml
[platform]
api-key = "YOUR_TELEGRAM_BOT_TOKEN"
# server = "https://custom.telegram.api/"  # optional, for custom Bot API server

# Map Telegram user IDs to their barcode ID strings.
# Users listed here get a merged barcode (their ID + input) on numeric input.
# Users not listed still get a single barcode on numeric input.
[users]
123456789 = "USER_A_CODE"
987654321 = "USER_B_CODE"
```

## Usage

```
courier [CONFIG]
```

`CONFIG` defaults to `config.toml`.

Set `RUST_LOG=info` (or `debug`) for logging output.

## Building

```
cargo build --release
```

## License

[![](https://www.gnu.org/graphics/agplv3-155x51.png "AGPL v3 logo")](https://www.gnu.org/licenses/agpl-3.0.txt)

Copyright (C) 2024-2026 KunoiSayami

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
