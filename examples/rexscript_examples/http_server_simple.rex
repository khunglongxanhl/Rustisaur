-- HTTP Server Simple Test - KHÔNG dùng HTTP client
rex.print("🌐 HTTP Server Simple Test")
rex.print("=" .. string.rep("=", 59))

-- Tạo server
local server = rex.http.server.create({
    host = "127.0.0.1",
    port = 8080,
    cors = true
})
rex.print("✅ Server created at " .. server.address)

-- Register handlers
local handlers = {}

handlers["GET:/"] = function(body)
    return rex.json.stringify({
        message = "Welcome to Rustisaur!",
        timestamp = rex.os.time()
    })
end

handlers["GET:/api/users"] = function(body)
    return rex.json.stringify({
        {id = 1, name = "Alice"},
        {id = 2, name = "Bob"}
    })
end

handlers["POST:/api/users"] = function(body)
    rex.print(" Received: " .. body)
    return rex.json.stringify({success = true})
end

rex.print("✅ Handlers registered")

-- Start server
rex.http.server.start(server)
rex.print("\n🚀 Server running at http://127.0.0.1:8080")
rex.print("📝 Mở terminal KHÁC và chạy:")
rex.print("   curl http://127.0.0.1:8080/")
rex.print("   curl http://127.0.0.1:8080/api/users")
rex.print("   Hoặc mở browser: http://127.0.0.1:8080/")
rex.print("\n⏹️  Press Ctrl+C to stop\n")

-- Main loop: poll and handle requests
while rex.http.server.is_running(server) do
    local req = rex.http.server.poll(server)
    
    if req then
        local key = req.method .. ":" .. req.path
        rex.print("📨 " .. key)
        
        local handler = handlers[key]
        
        if handler then
            local ok, result = pcall(handler, req.body)
            if ok then
                rex.http.server.respond(server, req.request_id, result, 200, "application/json")
                rex.print("  ✅ 200 OK")
            else
                rex.print("  ❌ Error: " .. tostring(result))
                rex.http.server.respond(server, req.request_id,
                    rex.json.stringify({error = tostring(result)}),
                    500, "application/json")
            end
        else
            rex.print("  ❌ 404 Not Found")
            rex.http.server.respond(server, req.request_id,
                rex.json.stringify({error = "Not Found"}),
                404, "application/json")
        end
    end
    
    rex.os.sleep(10)
end

rex.print("\n🛑 Server stopped")