-- ==========================================
-- BENCHMARK: Database Performance (SQLite)
-- ==========================================

rex.print("=== Database Benchmark ===🐘")

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

local ITERATIONS = 10000

-- ==========================================
-- SETUP: Create test table
-- ==========================================
rex.print("\n Setup:")
rex.store.db.execute("DROP TABLE IF EXISTS bench_test")
rex.store.db.execute([[
    CREATE TABLE bench_test (
        id INTEGER PRIMARY KEY,
        name TEXT,
        value REAL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )
]])
rex.print("✅ Created table: bench_test")

-- ==========================================
-- TEST 1: INSERT Performance
-- ==========================================
rex.print("\n INSERT Performance:")
benchmark("db.execute (INSERT)", function(i)
    rex.store.db.execute(
        "INSERT INTO bench_test (id, name, value) VALUES (?, ?, ?)",
        {i, "item_" .. i, i * 1.5}
    )
end, ITERATIONS)

-- ==========================================
-- TEST 2: SELECT Performance
-- ==========================================
rex.print("\n📖 SELECT Performance:")
benchmark("db.query (SELECT by ID)", function(i)
    rex.store.db.query(
        "SELECT * FROM bench_test WHERE id = ?",
        {i % ITERATIONS + 1}
    )
end, ITERATIONS)

benchmark("db.query_one (SELECT one)", function(i)
    rex.store.db.query_one(
        "SELECT * FROM bench_test WHERE id = ?",
        {i % ITERATIONS + 1}
    )
end, ITERATIONS)

-- ==========================================
-- TEST 3: UPDATE Performance
-- ==========================================
rex.print("\n️  UPDATE Performance:")
benchmark("db.execute (UPDATE)", function(i)
    rex.store.db.execute(
        "UPDATE bench_test SET value = ? WHERE id = ?",
        {i * 2.0, i % ITERATIONS + 1}
    )
end, ITERATIONS)

-- ==========================================
-- TEST 4: DELETE Performance
-- ==========================================
rex.print("\n🗑️  DELETE Performance:")
benchmark("db.execute (DELETE)", function(i)
    rex.store.db.execute(
        "DELETE FROM bench_test WHERE id = ?",
        {i % ITERATIONS + 1}
    )
end, ITERATIONS)

-- ==========================================
-- TEST 5: Transaction Performance
-- ==========================================
rex.print("\n💼 Transaction Performance:")
local batch_size = 100
local batches = ITERATIONS / batch_size

benchmark("Transaction (batch INSERT)", function(batch)
    rex.store.db.begin()
    for i = 1, batch_size do
        local id = (batch - 1) * batch_size + i
        rex.store.db.execute(
            "INSERT INTO bench_test (id, name, value) VALUES (?, ?, ?)",
            {id, "batch_" .. id, id * 1.5}
        )
    end
    rex.store.db.commit()
end, batches)

-- ==========================================
-- TEST 6: Complex Query Performance
-- ==========================================
rex.print("\n🔍 Complex Query Performance:")
benchmark("Aggregation query", function()
    rex.store.db.query([[
        SELECT 
            COUNT(*) as total,
            AVG(value) as avg_value,
            MAX(value) as max_value,
            MIN(value) as min_value
        FROM bench_test
    ]])
end, 100)

-- ==========================================
-- SUMMARY
-- ==========================================
local count = rex.store.db.query_one("SELECT COUNT(*) as total FROM bench_test")
rex.print("\n📊 Database Benchmark Summary:")
rex.print("Total rows: " .. count.total)

-- Cleanup
rex.store.db.execute("DROP TABLE bench_test")
rex.print("✅ Cleanup complete")