use rglua::interface;
use rglua::prelude::*;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedAuthor;
use serenity::all::CreateEmbedFooter;
use serenity::async_trait;
use tokio::runtime::Runtime;
use serenity::prelude::*;
use serenity::model::gateway::Ready;
use serenity::Client;
use serenity::model::channel::Message;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};
use lazy_static::lazy_static;
use std::thread;
use serenity::model::id::ChannelId;
use serenity::builder::CreateMessage;
use serenity::model::timestamp::Timestamp;
use serde_json::Value;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Clone)]
struct CachedMessage {
    id: String,
    content: String,
}

// Global channel for message passing
lazy_static! {
    static ref MESSAGE_CHANNEL: (Mutex<Sender<Message>>, Mutex<Receiver<Message>>) = {
        let (tx, rx) = channel();
        (Mutex::new(tx), Mutex::new(rx))
    };
    static ref RUNTIME: Mutex<Option<Runtime>> = Mutex::new(Some(Runtime::new().unwrap()));
    static ref BOT_TOKEN: Mutex<String> = Mutex::new(String::new());
    static ref BOT_LAUNCHED: Mutex<bool> = Mutex::new(false);
    static ref RESPONSE_CACHE: Mutex<HashMap<String, CachedMessage>> = Mutex::new(HashMap::new());
}

struct Handler {
    message_sender: Arc<Mutex<Sender<Message>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        if let Ok(mut bot_launched) = BOT_LAUNCHED.lock() {
            *bot_launched = true;
        }
    }


    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        
        // Send full message through channel
        if let Ok(sender) = self.message_sender.lock() {
            let _ = sender.send(msg);
        }
    }
}

#[lua_function]
fn process_discord_messages(l: LuaState) -> Result<i32, interface::Error> {
    if let Ok(receiver) = MESSAGE_CHANNEL.1.lock() {
        while let Ok(message) = receiver.try_recv() {
            // Get callback function
            let callback_name = std::ffi::CString::new("DiscordMessageCallback").unwrap();
            lua_getglobal(l, callback_name.as_ptr());
            
            // Create message table
            lua_newtable(l);
            
            // Add content
            lua_pushstring(l, cstr!("content"));
            let content_cstr = std::ffi::CString::new(message.content).unwrap();
            lua_pushstring(l, content_cstr.as_ptr());
            lua_settable(l, -3);
            
            // Add author info
            lua_pushstring(l, cstr!("author"));
            lua_newtable(l);
            
            // Author name
            lua_pushstring(l, cstr!("name"));
            let author_cstr = std::ffi::CString::new(message.author.name).unwrap();
            lua_pushstring(l, author_cstr.as_ptr());
            lua_settable(l, -3);
            
            // Author ID
            lua_pushstring(l, cstr!("id"));
            let author_id_str = std::ffi::CString::new(message.author.id.get().to_string()).unwrap();
            lua_pushstring(l, author_id_str.as_ptr());
            lua_settable(l, -3);
            
            // Author bot status
            lua_pushstring(l, cstr!("bot"));
            lua_pushboolean(l, if message.author.bot { 1 } else { 0 });
            lua_settable(l, -3);
            
            // [NEW] Add member roles if available (for guild messages)
            if let Some(member) = &message.member {
                lua_pushstring(l, cstr!("roles"));
                lua_newtable(l);
                for (i, role_id) in member.roles.iter().enumerate() {
                    lua_pushinteger(l, (i + 1) as isize);
                    let role_str = role_id.get().to_string();
                    let role_cstr = std::ffi::CString::new(role_str).unwrap();
                    lua_pushstring(l, role_cstr.as_ptr());
                    lua_settable(l, -3);
                }
                lua_settable(l, -3);
            }
            
            lua_settable(l, -3);
            
            // Add channel ID
            lua_pushstring(l, cstr!("channel_id"));
            let channel_id_str = std::ffi::CString::new(message.channel_id.get().to_string()).unwrap();
            lua_pushstring(l, channel_id_str.as_ptr());
            lua_settable(l, -3);
            
            // Add message ID
            lua_pushstring(l, cstr!("message_id"));
            let message_id_str = std::ffi::CString::new(message.id.get().to_string()).unwrap();
            lua_pushstring(l, message_id_str.as_ptr());
            lua_settable(l, -3);
            
            // Add timestamp
            lua_pushstring(l, cstr!("timestamp"));
            let timestamp_str = std::ffi::CString::new(message.timestamp.to_string()).unwrap();
            lua_pushstring(l, timestamp_str.as_ptr());
            lua_settable(l, -3);
            
            // Add is_own flag
            lua_pushstring(l, cstr!("is_own"));
            lua_pushboolean(l, if message.webhook_id.is_none() && !message.author.bot { 1 } else { 0 });
            lua_settable(l, -3);

            // Call the callback with the table
            lua_call(l, 1, 0);
        }
    }
    
    Ok(0)
}

#[lua_function]
fn connect_discord_bot(l: LuaState) -> Result<i32, interface::Error> {
    // Get token argument
    let token = luaL_checkstring(l, 1);
    let token_owned = rstr!(token).to_string();
    
    // Store token for later use
    if let Ok(mut token_guard) = BOT_TOKEN.lock() {
        *token_guard = token_owned.clone();
    }
    
    // Get callback reference
    // Spawn a new thread for Discord bot
    thread::spawn(move || {
        if let Ok(rt_opt) = RUNTIME.lock() {
            if let Some(rt) = rt_opt.as_ref() {
                // Create and spawn the client in a non-blocking way
                rt.spawn(async move {
                    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
                    
                    let handler = Handler {
                        message_sender: Arc::new(Mutex::new(MESSAGE_CHANNEL.0.lock().unwrap().clone())),
                    };
                    
                    match Client::builder(&token_owned, intents)
                        .event_handler(handler)
                        .await {
                            Ok(mut client) => {
                                if let Err(why) = client.start().await {
                                    println!("Client error: {:?}", why);
                                }
                            },
                            Err(e) => println!("Error creating client: {:?}", e)
                        };
                });
            }
        }
    });

    // Return immediately without blocking
    lua_pushstring(l, cstr!("Bot connecting in background..."));
    
    Ok(1)
}

#[lua_function]
fn send_discord_message(l: LuaState) -> Result<i32, interface::Error> {
    let channel_id_str = luaL_checkstring(l, 1);
    let channel_id = rstr!(channel_id_str).parse::<u64>().unwrap_or(0);
    
    let message = luaL_checkstring(l, 2);
    let message_owned = rstr!(message).to_string();
    
    let request_id = luaL_checkstring(l, 3);
    let request_id_owned = rstr!(request_id).to_string();
    
    let token = BOT_TOKEN.lock().unwrap().clone();
    
    thread::spawn(move || {
        if let Ok(rt_opt) = RUNTIME.lock() {
            if let Some(rt) = rt_opt.as_ref() {
                rt.spawn(async move {
                    let channel = ChannelId::new(channel_id);
                    match channel.say(&serenity::http::Http::new(&token), message_owned).await {
                        Ok(response) => {
                            let cached = CachedMessage {
                                id: response.id.get().to_string(),
                                content: response.content.clone(),
                            };
                            let mut cache = RESPONSE_CACHE.lock().unwrap();
                            cache.insert(request_id_owned, cached);
                        },
                        Err(why) => println!("Error sending message: {:?}", why),
                    }
                });
            }
        }
    });

    lua_pushstring(l, cstr!("Message queued"));
    Ok(1)
}

#[lua_function]
fn send_rich_message(l: LuaState) -> Result<i32, interface::Error> {
    let channel_id_str = luaL_checkstring(l, 1);
    let channel_id = rstr!(channel_id_str).parse::<u64>().unwrap_or(0);
    
    let json_data = luaL_checkstring(l, 2);
    let json_data_owned = rstr!(json_data).to_string();
    
    let token = BOT_TOKEN.lock().unwrap().clone();

    thread::spawn(move || {
        if let Ok(rt_opt) = RUNTIME.lock() {
            if let Some(rt) = rt_opt.as_ref() {
                rt.spawn(async move {
                    let channel = ChannelId::new(channel_id);
                    let json_value: Value = serde_json::from_str(&json_data_owned).unwrap_or_default();
                    
                    let footer = CreateEmbedFooter::new(json_value.get("footer").and_then(|v| v.as_str()).unwrap_or(""));
                    let embed = CreateEmbed::new()
                        .title(json_value.get("title").and_then(|v| v.as_str()).unwrap_or(""))
                        .description(json_value.get("description").and_then(|v| v.as_str()).unwrap_or(""))
                        .image(json_value.get("image").and_then(|v| v.as_str()).unwrap_or(""))
                        .fields(
                            json_value.get("fields").and_then(|v| v.as_array()).unwrap_or(&Vec::new())
                                .iter()
                                .map(|field| (
                                    field.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                                    field.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                                    field.get("inline").and_then(|v| v.as_bool()).unwrap_or(false)
                                ))
                                .collect::<Vec<_>>()
                        )
                        .footer(footer)
                        .thumbnail(json_value.get("thumbnail").and_then(|v| v.as_str()).unwrap_or(""))
                        .color(json_value.get("color").and_then(|v| v.as_u64()).unwrap_or(0))
                        .author(CreateEmbedAuthor::new(json_value.get("author").and_then(|v| v.as_str()).unwrap_or("")))
                        .timestamp(Timestamp::now());
                    let builder = CreateMessage::new().embed(embed);
                    if let Err(why) = channel.send_message(&serenity::http::Http::new(&token), builder).await {
                        println!("Error sending message: {:?}", why);
                    }
                });
            }
        }
    });

    lua_pushstring(l, cstr!("Rich message queued"));
    Ok(1)
}

#[lua_function]
fn random_uuid(l: LuaState) -> Result<i32, interface::Error> {
    let uuid = Uuid::new_v4().to_string();
    lua_pushstring(l, uuid.as_ptr() as *const i8);
    Ok(1)
}

#[lua_function]
fn delete_discord_message(l: LuaState) -> Result<i32, interface::Error> {
    // Get channel ID as string
    let channel_id_str = luaL_checkstring(l, 1);
    let channel_id = rstr!(channel_id_str).parse::<u64>().unwrap_or(0);
    
    // Get message ID as string
    let message_id_str = luaL_checkstring(l, 2);
    let message_id = rstr!(message_id_str).parse::<u64>().unwrap_or(0);
    
    // Get stored token
    let token = BOT_TOKEN.lock().unwrap().clone();
    
    thread::spawn(move || {
        if let Ok(rt_opt) = RUNTIME.lock() {
            if let Some(rt) = rt_opt.as_ref() {
                rt.spawn(async move {
                    let channel = ChannelId::new(channel_id);
                    if let Err(why) = channel.delete_message(&serenity::http::Http::new(&token), message_id).await {
                        println!("Error deleting message: {:?}", why);
                    }
                });
            }
        }
    });

    lua_pushstring(l, cstr!("Message deletion queued"));
    Ok(1)
}

#[lua_function]
fn set_user_role(l: LuaState) -> Result<i32, interface::Error> {
    // Get guild ID as string
    let guild_id_str = luaL_checkstring(l, 1);
    let guild_id = rstr!(guild_id_str).parse::<u64>().unwrap_or(0);
    
    // Get user ID as string
    let user_id_str = luaL_checkstring(l, 2);
    let user_id = rstr!(user_id_str).parse::<u64>().unwrap_or(0);
    
    // Get role ID as string
    let role_id_str = luaL_checkstring(l, 3);
    let role_id = rstr!(role_id_str).parse::<u64>().unwrap_or(0);
    
    // Get stored token
    let token = BOT_TOKEN.lock().unwrap().clone();
    
    thread::spawn(move || {
        if let Ok(rt_opt) = RUNTIME.lock() {
            if let Some(rt) = rt_opt.as_ref() {
                rt.spawn(async move {
                    let guild = serenity::model::id::GuildId::new(guild_id);
                    let user = serenity::model::id::UserId::new(user_id);
                    if let Err(why) = async {
                        let member = guild.member(&serenity::http::Http::new(&token), user).await?;
                        member.add_role(&serenity::http::Http::new(&token), role_id).await
                    }.await {
                        println!("Error setting user role: {:?}", why);
                    }
                });
            }
        }
    });

    lua_pushstring(l, cstr!("Role assignment queued"));
    Ok(1)
}

#[lua_function]
fn bot_launched(l: LuaState) -> Result<i32, interface::Error> {
    let bot_launched = BOT_LAUNCHED.lock().unwrap().clone();
    lua_pushboolean(l, if bot_launched { 1 } else { 0 });
    Ok(1)
}

#[lua_function]
fn get_response(l: LuaState) -> Result<i32, interface::Error> {
    let request_id = luaL_checkstring(l, 1);
    let request_id_owned = rstr!(request_id).to_string();
    
    let cache = match RESPONSE_CACHE.lock() {
        Ok(cache) => cache,
        Err(poisoned) => {
            println!("Cache lock poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    if let Some(response) = cache.get(&request_id_owned) {
        lua_newtable(l);
        
        lua_pushstring(l, cstr!("id"));
        let message_id_cstr = std::ffi::CString::new(response.id.clone()).unwrap();
        lua_pushstring(l, message_id_cstr.as_ptr());
        lua_settable(l, -3);
        
        lua_pushstring(l, cstr!("content"));
        let content_cstr = std::ffi::CString::new(response.content.clone()).unwrap();
        lua_pushstring(l, content_cstr.as_ptr());
        lua_settable(l, -3);
        
        Ok(1)
    } else {
        lua_pushnil(l);
        Ok(1)
    }
}

fn clear_cache() {
    let mut cache = RESPONSE_CACHE.lock().unwrap();
    cache.clear();
}

#[gmod_open]
fn open(l: LuaState) -> Result<i32, interface::Error> {
    printgm!(l, "Loaded engine module!");
    println!("loaded");

    // Register functions
    let lib = reg! [
        "connect_discord_bot" => connect_discord_bot,
        "process_discord_messages" => process_discord_messages,
        "send_message" => send_discord_message,
        "send_rich_message" => send_rich_message,
        "delete_discord_message" => delete_discord_message,
        "set_user_role" => set_user_role,
        "bot_launched" => bot_launched,
        "get_response" => get_response
    ];

    luaL_register(l, cstr!("discord"), lib.as_ptr());

    // interface with util functions
    let lib2 = reg! [
        "random_uuid" => random_uuid
    ];

    luaL_register(l, cstr!("util"), lib2.as_ptr());

    Ok(0)
}

#[gmod_close]
fn close(_l: LuaState) -> i32 {
    // Stop the bot gracefully before the library is unloaded.
    if let Ok(mut rt_opt) = RUNTIME.lock() {
        if let Some(rt) = rt_opt.take() {
            rt.shutdown_timeout(std::time::Duration::from_secs(5));
        }
    }
    
    clear_cache();
    0
}
