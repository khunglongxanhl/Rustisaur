local json_str = '{"name": "Rustisaur", "version": 1}'
local data = rex.json.parse(json_str)
rex.print("Name: " .. data.name)
rex.print("Version: " .. data.version)