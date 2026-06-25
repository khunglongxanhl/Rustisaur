-- ==========================================
-- BENCHMARK: String Operations Performance
-- ==========================================

rex.print("=== String Benchmark ===📝")

-- Helper function
local function benchmark(name, func, iterations)
    local start = os.clock()
    for i = 1, iterations do
        func(i)
    end
    local elapsed = os.clock() - start
    local ops_per_sec = iterations / (elapsed > 0 and elapsed or 0.001)
    rex.print(string.format("%-30s: %d ops in %.3fs (%.0f ops/sec)", 
        name, iterations, elapsed, ops_per_sec))
    return elapsed
end

local ITERATIONS = 100000
local TEST_STRING = "Hello, Rustisaur! This is a benchmark test."

-- ==========================================
-- TEST 1: Case Conversion
-- ==========================================
rex.print("\n Case Conversion:")
benchmark("string.upper", function()
    rex.string.upper(TEST_STRING)
end, ITERATIONS)

benchmark("string.lower", function()
    rex.string.lower(TEST_STRING)
end, ITERATIONS)

benchmark("string.capitalize", function()
    rex.string.capitalize(TEST_STRING)
end, ITERATIONS)

-- ==========================================
-- TEST 2: Trim Operations
-- ==========================================
rex.print("\n✂️  Trim Operations:")
local padded_string = "   " .. TEST_STRING .. "   "

benchmark("string.trim", function()
    rex.string.trim(padded_string)
end, ITERATIONS)

benchmark("string.trim_left", function()
    rex.string.trim_left(padded_string)
end, ITERATIONS)

benchmark("string.trim_right", function()
    rex.string.trim_right(padded_string)
end, ITERATIONS)

-- ==========================================
-- TEST 3: Search & Replace
-- ==========================================
rex.print("\n🔍 Search & Replace:")
benchmark("string.contains", function()
    rex.string.contains(TEST_STRING, "Rustisaur")
end, ITERATIONS)

benchmark("string.starts_with", function()
    rex.string.starts_with(TEST_STRING, "Hello")
end, ITERATIONS)

benchmark("string.ends_with", function()
    rex.string.ends_with(TEST_STRING, "test.")
end, ITERATIONS)

benchmark("string.replace", function()
    rex.string.replace(TEST_STRING, "Rustisaur", "World")
end, ITERATIONS)

benchmark("string.replace_all", function()
    rex.string.replace_all(TEST_STRING, "e", "E")
end, ITERATIONS)

-- ==========================================
-- TEST 4: Split & Join
-- ==========================================
rex.print("\n🔗 Split & Join:")
benchmark("string.split", function()
    rex.string.split(TEST_STRING, " ")
end, ITERATIONS)

local parts = rex.string.split(TEST_STRING, " ")
benchmark("string.join", function()
    rex.string.join(parts, "-")
end, ITERATIONS)

-- ==========================================
-- TEST 5: Transform Operations
-- ==========================================
rex.print("\n🔄 Transform Operations:")
benchmark("string.reverse", function()
    rex.string.reverse(TEST_STRING)
end, ITERATIONS)

benchmark("string.repeat", function()
    rex.string.repeat("abc", 10)
end, ITERATIONS)

benchmark("string.slice", function()
    rex.string.slice(TEST_STRING, 0, 10)
end, ITERATIONS)

-- ==========================================
-- TEST 6: Length & Check
-- ==========================================
rex.print("\n📏 Length & Check:")
benchmark("string.len", function()
    rex.string.len(TEST_STRING)
end, ITERATIONS)

benchmark("string.is_empty", function()
    rex.string.is_empty(TEST_STRING)
end, ITERATIONS)

-- ==========================================
-- SUMMARY
-- ==========================================
rex.print("\n📊 String Benchmark Summary:")
rex.print("Test string length: " .. rex.string.len(TEST_STRING))
rex.print("All operations completed successfully!")