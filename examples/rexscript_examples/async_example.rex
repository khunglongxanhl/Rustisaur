-- Async example using Rustisaur async file I/O
rex.print("Reading file asynchronously...")

local content = rex.fs.read_async("Cargo.toml")
rex.print("File size: " .. #content .. " bytes")
rex.print.success("Async read complete")
