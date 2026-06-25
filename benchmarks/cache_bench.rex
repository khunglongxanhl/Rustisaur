-- ==========================================
-- BENCHMARK: Cache Performance (Redis-like)
-- ==========================================

rex.print("🔥=== Cache Benchmark ===🔥")

-- Helper function để đo thời gian
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

-- ==========================================
-- TEST 1: Write Performance
-- ==========================================
rex.print("\n📝 Write Performance:")
benchmark("cache.set (no TTL)", function(i)
    rex.store.cache.set("key:" .. i, "value:" .. i)
end, ITERATIONS)

benchmark("cache.set (with TTL)", function(i)
    rex.store.cache.set("key_ttl:" .. i, "value:" .. i, 3600)
end, ITERATIONS)

-- ==========================================
-- TEST 2: Read Performance
-- ==========================================
rex.print("\n📖 Read Performance:")
benchmark("cache.get (hit)", function(i)
    rex.store.cache.get("key:" .. (i % 1000))
end, ITERATIONS)

benchmark("cache.get (miss)", function(i)
    rex.store.cache.get("nonexistent:" .. i)
end, ITERATIONS)

-- ==========================================
-- TEST 3: Increment Performance
-- ==========================================
rex.print("\n Increment Performance:")
rex.store.cache.set("counter", 0)
benchmark("cache.incr", function()
    rex.store.cache.incr("counter")
end, ITERATIONS)

-- ==========================================
-- TEST 4: Exists Performance
-- ==========================================
rex.print("\n✅ Exists Performance:")
benchmark("cache.exists (true)", function(i)
    rex.store.cache.exists("key:" .. (i % 1000))
end, ITERATIONS)

benchmark("cache.exists (false)", function(i)
    rex.store.cache.exists("nonexistent:" .. i)
end, ITERATIONS)

-- ==========================================
-- TEST 5: Delete Performance
-- ==========================================
rex.print("\n🗑️  Delete Performance:")
-- Create keys to delete
for i = 1, ITERATIONS do
    rex.store.cache.set("to_delete:" .. i, "value")
end

benchmark("cache.delete", function(i)
    rex.store.cache.delete("to_delete:" .. i)
end, ITERATIONS)

-- ==========================================
-- SUMMARY
-- ==========================================
rex.print("\n📊 Cache Benchmark Summary:")
rex.print("Total keys in cache: " .. rex.store.cache.len())
rex.print("Cache is empty: " .. tostring(rex.store.cache.is_empty()))

-- Cleanup
rex.store.cache.clear()
rex.print("✅ Cache cleared")