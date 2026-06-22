-- Discord bot example (conceptual)
-- Requires a Discord library integration (planned for future releases)

rex.print("Discord Bot Example")
rex.print.warn("Full Discord integration is planned for v0.2.0")

local token = rex.env.get("DISCORD_TOKEN")
if not token then
    rex.print.error("Set DISCORD_TOKEN environment variable")
    return
end

rex.print.success("Bot framework ready - Discord gateway support coming soon")

-- Event-driven architecture demo
local events = rex.event.create()

events:on("message", function(data)
    rex.print("Message from " .. (data.author or "unknown"))
end)

events:emit("message", {author = "demo_user", content = "Hello!"})
