-- Test Guardian Network Firewall
rex.print("🛡️  Guardian Security Test")

-- Test 1: Check allowed domain
rex.print("\n--- Test 1: Check google.com ---")
local allowed = rex.guardian.check_network("https://www.google.com")
rex.print("Allowed: " .. tostring(allowed))

-- Test 2: Check blocked domain
rex.print("\n--- Test 2: Check malware.com ---")
allowed = rex.guardian.check_network("https://malware.com/steal-data")
rex.print("Allowed: " .. tostring(allowed))

-- Test 3: Add to whitelist
rex.print("\n--- Test 3: Add api.example.com to whitelist ---")
rex.guardian.allow_domain("api.example.com")
allowed = rex.guardian.check_network("https://api.example.com/data")
rex.print("Allowed: " .. tostring(allowed))

-- Test 4: Add to blacklist
rex.print("\n--- Test 4: Block suspicious.com ---")
rex.guardian.block_domain("suspicious.com")
allowed = rex.guardian.check_network("https://suspicious.com/attack")
rex.print("Allowed: " .. tostring(allowed))

-- Test 5: Show stats
rex.print("\n--- Test 5: Guardian Statistics ---")
rex.guardian.show_stats()

local stats = rex.guardian.stats()
rex.print("\nTotal: " .. stats.total_requests)
rex.print("Allowed: " .. stats.allowed_requests)
rex.print("Blocked: " .. stats.blocked_requests)

rex.print("\n✅ Guardian test complete!")