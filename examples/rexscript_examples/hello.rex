-- Hello World Rustisaur
rex.print("Hello, Rustisaur!")
rex.print.success("Welcome to the Rustisaur scripting language")

local name = rex.input("Enter your name: ")
if name ~= "" then
    rex.print("Hello, " .. name .. "!")
end
