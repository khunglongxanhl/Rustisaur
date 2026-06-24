-- ==========================================
-- TEST: rex.store (Hybrid Storage)
-- Cache (Redis-like) + Database (SQLite)
-- ==========================================

rex.print("🦖=== Rustisaur Storage Test ===🦖")

-- ==========================================
-- 1. TEST CACHE (TỐC ĐỘ PHẢN LỰC)
-- ==========================================
rex.print("\n🔥--- Cache (Redis-like) ---🔥")

-- Set với TTL 60 giây
rex.store.cache.set("user:123", {name = "John", score = 100}, 60)
rex.print("✅ Set cache: user:123")

-- Get value
local user = rex.store.cache.get("user:123")
if user then
    rex.print("✅ Get: " .. user.name .. " - Score: " .. user.score)
else
    rex.print("❌ Failed to get cache")
end

-- Increment counter
rex.store.cache.incr("counter")
rex.store.cache.incr("counter")
rex.store.cache.incr("counter")
local count = rex.store.cache.get("counter")
rex.print("✅ Counter after 3x incr: " .. tostring(count))

-- List keys
local keys = rex.store.cache.keys()
rex.print("✅ Total keys in cache: " .. rex.store.cache.len())

-- ==========================================
-- 2. TEST DATABASE (SQLITE - HARD TASKS)
-- ==========================================
rex.print("\n🐘--- Database (SQLite) ---🐘")

-- Create table
rex.store.db.execute([[
    CREATE TABLE IF NOT EXISTS players (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        score INTEGER DEFAULT 0,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP
    )
]])
rex.print("✅ Created table: players")

-- Clear old data (for clean test)
rex.store.db.execute("DELETE FROM players")

-- Insert data với params
rex.store.db.execute("INSERT INTO players (id, name, score) VALUES (?, ?, ?)", {1, "Alice", 100})
rex.store.db.execute("INSERT INTO players (id, name, score) VALUES (?, ?, ?)", {2, "Bob", 85})
rex.store.db.execute("INSERT INTO players (id, name, score) VALUES (?, ?, ?)", {3, "Charlie", 92})
rex.print("✅ Inserted 3 players")

-- Query all
rex.print("\n📊 All Players (ordered by score DESC):")
local players = rex.store.db.query("SELECT * FROM players ORDER BY score DESC")
for i, player in ipairs(players) do
    rex.print("  " .. i .. ". " .. player.name .. " - Score: " .. player.score)
end

-- Query with WHERE
rex.print("\n🏆 Top Players (score > 90):")
local top = rex.store.db.query(
    "SELECT name, score FROM players WHERE score > ? ORDER BY score DESC",
    {90}
)
for i, p in ipairs(top) do
    rex.print("  " .. i .. ". " .. p.name .. " - Score: " .. p.score)
end

-- Query one
local alice = rex.store.db.query_one("SELECT * FROM players WHERE name = ?", {"Alice"})
if alice then
    rex.print("\n👤 Found Alice: Score = " .. alice.score)
end

-- Transaction test
rex.print("\n💼 Transaction Test:")
rex.store.db.begin()
rex.store.db.execute("UPDATE players SET score = score + 10 WHERE name = ?", {"Alice"})
rex.store.db.execute("UPDATE players SET score = score + 5 WHERE name = ?", {"Bob"})
rex.store.db.commit()
rex.print("✅ Transaction committed")

-- Verify
local updated = rex.store.db.query("SELECT name, score FROM players ORDER BY name")
for _, p in ipairs(updated) do
    rex.print("  " .. p.name .. ": " .. p.score)
end

-- Count
local count_row = rex.store.db.query_one("SELECT COUNT(*) as total FROM players")
rex.print("\n📈 Total players: " .. count_row.total)

rex.print("\n🦖=== All tests passed! ===🦖")
rex.print("💾 Database file: rustisaur.db")