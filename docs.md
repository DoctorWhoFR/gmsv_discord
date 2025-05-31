# Discord Bot Lua API Documentation

This document provides an overview of the Lua functions available for interacting with the Discord bot.

## Functions

### `connect_discord_bot`
fzefezfezfezfzeffff
Connects the Discord bot using the provided token.

- **Parameters**:
  - `token` (string): The bot token.

- **Example**:
  ```lua
  discord.connect_discord_bot("YOUR_BOT_TOKEN")
  ```

### `process_discord_messages`

Processes incoming Discord messages and triggers the `DiscordMessageCallback` in Lua.

- **Parameters**: None

- **Example**:
  ```lua
  discord.process_discord_messages()
  ```

### `send_message`

Sends a plain text message to a specified channel.

- **Parameters**:
  - `channel_id` (string): The ID of the channel.
  - `message` (string): The message content.

- **Example**:
  ```lua
  discord.send_message("123456789012345678", "Hello, Discord!")
  ```

### `send_rich_message`

Sends a rich embed message to a specified channel using JSON data.

- **Parameters**:
  - `channel_id` (string): The ID of the channel.
  - `json_data` (string): JSON string containing embed details.

- **Example**:
  ```lua
  local json_data = [[
  {
    "title": "Embed Title",
    "description": "This is an embed description.",
    "color": 16711680
  }
  ]]
  discord.send_rich_message("123456789012345678", json_data)
  ```

### `delete_discord_message`

Deletes a message from a specified channel.

- **Parameters**:
  - `channel_id` (string): The ID of the channel.
  - `message_id` (string): The ID of the message to delete.

- **Example**:
  ```lua
  discord.delete_discord_message("123456789012345678", "987654321098765432")
  ```

### `set_user_role`

Assigns a role to a user in a specified guild.

- **Parameters**:
  - `guild_id` (string): The ID of the guild.
  - `user_id` (string): The ID of the user.
  - `role_id` (string): The ID of the role to assign.

- **Example**:
  ```lua
  discord.set_user_role("123456789012345678", "234567890123456789", "345678901234567890")
  ```

### `random_uuid`

Generates a random UUID.

- **Parameters**: None

- **Example**:
  ```lua
  local uuid = util.random_uuid()
  print("Generated UUID: " .. uuid)
  ```

## Notes

- Ensure that the bot has the necessary permissions to perform actions such as sending messages, deleting messages, and managing roles.
- The `process_discord_messages` function should be called regularly to handle incoming messages and trigger Lua callbacks. 