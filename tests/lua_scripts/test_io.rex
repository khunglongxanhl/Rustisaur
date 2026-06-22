-- I/O test script
local test_file = "_rex_io_test.txt"
rex.fs.write(test_file, "io test content")
local content = rex.fs.read(test_file)
assert(content == "io test content")
os.remove(test_file)
rex.print.success("I/O test passed")
