# DiscordBot

DiscordBot is a Rust-based module for integrating Discord bot functionality with Garry's Mod using the Serenity library. It allows you to send and receive messages, including rich embeds, from within the game.

## Features
ffezfezffff
- Connect a Discord bot to a server.
- Send and receive messages.
- Send rich embeds with customizableddd fields.fff
- Generate random UUIDs.fezfzefezfezfezffffffffff
fezfezfezfezfzefez
## Dependencies

- [Serenity](https://github.com/serenity-rs/serenity) for Discord API integration.
- [Tokio](https://tokio.rs/) for asynchronous runtime.
- [Lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs) for static variables.
- [Serde_json](https://github.com/serde-rs/json) for JSON parsing.
- [UUID](https://github.com/uuid-rs/uuid) for generating random UUIDs.
ffff
## Installation

1. Ensure you have Rust and Cargo installed. You can download them from [rust-lang.org](https://www.rust-lang.org/).

2. Clone the repository:

   ```bash
   git clone https://github.com/DoctorWhoFR/gmsv_discord.git
   cd discordbot
   ```

<h2>🔨 Build project</h2>

To build the sample project in debug mode, you need to specify the target architecture for your build.

| Platform  |                     Command                     |                                                          Description                                                           |
|:---------:|:-----------------------------------------------:|:------------------------------------------------------------------------------------------------------------------------------:|
|  `win32`  |   `cargo build --target i686-pc-windows-msvc`   | Windows 32-bit<br>Use this if your server is running Windows on the `main` branch of Garry's Mod (this is the default branch). |
|  `win64`  |  `cargo build --target x86_64-pc-windows-msvc`  |              Windows 64-bit<br>Use this if your server is running Windows on the `x86-64` branch of Garry's Mod.               |
|  `linux`  |  `cargo build --target i686-unknown-linux-gnu`  |   Linux 32-bit<br>Use this if your server is running Linux on the `main` branch of Garry's Mod (this is the default branch).   |
| `linux64` | `cargo build --target x86_64-unknown-linux-gnu` |                Linux 64-bit<br>Use this if your server is running Linux on the `x86-64` branch of Garry's Mod.                 |

If Rust reports it cannot find the target/toolchain, you may need to install it. By default, Rust installs the native
toolchain for your system, which is likely Windows 64-bit (`x86_64-pc-windows-msvc`).

Cross-compiling Linux binaries on Windows is not recommended. For compiling Linux binaries on Windows, use WSL.

rename the compiled binary to `gmsv_discordbot_PLATFORM.dll`

Next, move the renamed binary to `garrysmod/lua/bin/` on your server. If the `bin` folder does not exist, create it.

Finally, you can load the module from Lua with:
```lua
require("discordbot")
```

## Usage


- Load the compiled library into Garry's Mod.
- Use the provided Lua functions to interact with Discord:
  - `discord.connect_discord_bot(token)`: Connects the bot using the provided token.
  - `discord.send_message(channel_id, message)`: Sends a message to the specified channel.
  - `discord.send_rich_message(channel_id, json_data)`: Sends a rich embed message using JSON data.
  - `discord.random_uuid()`: Generates a random UUID.


- `process_discord_messages()`: Processes incoming Discord messages. (Use it in a timer), it will also retrieve the global function DiscordMessageCallback that you need to create like : 

```lua

function DiscordMessageCallback(message)
    PrintTable(message)
end

timer.Create("process_discord_messages", 1, 0, function()
    discord.process_discord_messages()
end)

```

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request for any improvements or bug fixes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
