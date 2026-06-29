-- Test WebSocket Client - Polling Pattern
rex.print("🔌 WebSocket Client Test (Polling Pattern)")
rex.print("=" .. string.rep("=", 59))

-- Test 1: Connect to echo server
rex.print("\n📡 Test 1: Connect to echo server")
local ws = rex.websocket.connect("wss://echo.websocket.org")

-- Wait for connection
rex.os.sleep(1000)

rex.print("State: " .. rex.websocket.get_state(ws))
rex.print("Connected: " .. tostring(rex.websocket.is_connected(ws)))

-- Check for connection errors
local errors = rex.websocket.poll_errors(ws)
if #errors > 0 then
    rex.print("❌ Connection errors:")
    for _, err in ipairs(errors) do
        rex.print("  - " .. err)
    end
end

if not rex.websocket.is_connected(ws) then
    rex.print("❌ Not connected, exiting test")
    return
end

-- Test 2: Send and receive messages (polling pattern)
rex.print("\n📡 Test 2: Send and poll messages")

-- Send messages
rex.websocket.send(ws, "Hello WebSocket!")
rex.os.sleep(200)

rex.websocket.send(ws, "Rustisaur is awesome!")
rex.os.sleep(200)

-- Poll for responses
rex.print("\n📥 Polling for messages...")
for i = 1, 10 do
    local msg = rex.websocket.poll(ws)
    if msg then
        rex.print("  Received: " .. msg)
    else
        rex.print("  (no message)")
    end
    rex.os.sleep(200)
end

-- Test 3: Send JSON
rex.print("\n📡 Test 3: Send JSON")
local json_msg = rex.json.stringify({
    type = "test",
    data = {
        message = "Hello from Rustisaur",
        timestamp = rex.os.time()
    }
})
rex.websocket.send(ws, json_msg)
rex.os.sleep(500)

-- Poll for response
local response = rex.websocket.poll(ws)
if response then
    rex.print("📥 Received JSON response: " .. response)
    local parsed = rex.json.parse(response)
    if parsed then
        rex.print("  Type: " .. tostring(parsed.type))
        rex.print("  Message: " .. tostring(parsed.data.message))
    end
end

-- Test 4: Batch poll
rex.print("\n📡 Test 4: Batch poll")
rex.websocket.send(ws, "Message 1")
rex.websocket.send(ws, "Message 2")
rex.websocket.send(ws, "Message 3")
rex.os.sleep(500)

local all_messages = rex.websocket.poll_all(ws)
rex.print("Received " .. #all_messages .. " messages:")
for i, msg in ipairs(all_messages) do
    rex.print("  " .. i .. ": " .. msg)
end

-- Test 5: Check final state
rex.print("\n📡 Test 5: Final state")
rex.print("State: " .. rex.websocket.get_state(ws))
rex.print("Is connected: " .. tostring(rex.websocket.is_connected(ws)))

-- Test 6: Close connection
rex.print("\n📡 Test 6: Close connection")
rex.websocket.close(ws)
rex.os.sleep(200)
rex.print("State after close: " .. rex.websocket.get_state(ws))

rex.print("\n✅ WebSocket test complete!")