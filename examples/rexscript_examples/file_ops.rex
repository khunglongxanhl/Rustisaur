-- File operations example
local filename = "rustisaur_test.txt"

rex.fs.write(filename, "Rustisaur file I/O test\n")
rex.print("Wrote to " .. filename)

local content = rex.fs.read(filename)
rex.print("Content: " .. content)

rex.fs.append(filename, "Appended line\n")
rex.print.success("File operations complete")

-- Cleanup
if rex.fs.exists(filename) then
    os.remove(filename)
end
