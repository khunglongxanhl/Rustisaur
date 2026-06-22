# Rustisaur Lua API Reference

Rustisaur scripts use Lua 5.4 syntax. All Rustisaur APIs are exposed via the global `rex` table.

## Console I/O

```lua
rex.print("Hello, World!")
rex.print.error("Error message")
rex.print.success("Success message")
rex.print.warn("Warning message")

local input = rex.input("Enter your name: ")
local password = rex.input.password("Password: ")
```

## File I/O

```lua
local content = rex.fs.read("file.txt")
rex.fs.write("output.txt", "Hello")
rex.fs.append("log.txt", "New line")
local exists = rex.fs.exists("file.txt")

-- Async
local content = rex.fs.read_async("file.txt")
rex.fs.write_async("output.txt", "Hello")
```

## HTTP Client

```lua
local response = rex.http.get("https://api.example.com/data")
rex.print(response.status)
rex.print(response.body)

local post_response = rex.http.post("https://api.example.com/submit", {
    body = rex.json.stringify({name = "test"}),
    headers = {["Authorization"] = "Bearer token"}
})
```

## JSON

```lua
local obj = rex.json.parse('{"name": "John", "age": 30}')
local str = rex.json.stringify({name = "Jane", age = 25})
local pretty = rex.json.stringify_pretty({a = 1, b = 2})
```

## Events

```lua
local emitter = rex.event.create()
emitter:on("user:login", function(data)
    rex.print("User: " .. data.username)
end)
emitter:emit("user:login", {username = "john", id = 123})
```

## Environment & Process

```lua
local path = rex.env.get("PATH")
rex.env.set("MY_VAR", "value")
local cwd = rex.process.cwd()
local pid = rex.process.pid()
```

## Time

```lua
local now = rex.time.now()
rex.print(now:unix())
rex.print(now:format("%Y-%m-%d %H:%M:%S"))
```

## Math

```lua
rex.print(rex.math.random(1, 100))
rex.print(rex.math.min(1, 2, 3))
rex.print(rex.math.max(1, 2, 3))
```

## String Utilities

```lua
rex.print(rex.string.upper("hello"))
rex.print(rex.string.lower("WORLD"))
rex.print(rex.string.split("a,b,c", ","))
rex.print(rex.string.trim("  hello  "))
```

## Table Utilities

```lua
local t = {1, 2, 3}
rex.table.push(t, 4)
rex.print(rex.table.length(t))
rex.table.remove(t, 2)
```
