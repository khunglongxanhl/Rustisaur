-- Test Hello World
rex.print("=== Test Rustisaur ===")
rex.print("Hello, World!")

-- Test string functions
local name = "  Rustisaur  "
rex.print("Original: " .. name)
rex.print("Trimmed: " .. rex.string.trim(name))
rex.print("Upper: " .. rex.string.upper(name))

-- Test math functions
rex.print("\nMath tests:")
rex.print("sqrt(16) = " .. rex.math.sqrt(16))
rex.print("max(3, 7, 2) = " .. rex.math.max(3, 7, 2))

-- Test file operations
rex.print("\nFile test:")
rex.fs.write("temp.txt", "Hello from Rustisaur!")
local content = rex.fs.read("temp.txt")
rex.print("File content: " .. content)
rex.fs.delete("temp.txt")
rex.print("File deleted!")

rex.print("\n=== All tests passed! ===")