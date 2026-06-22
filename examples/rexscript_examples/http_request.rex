-- HTTP request example (requires network)
rex.print("Fetching https://httpbin.org/get ...")

local response = rex.http.get("https://httpbin.org/get")
rex.print("Status: " .. response.status)

local data = rex.json.parse(response.body)
if data.url then
    rex.print("URL: " .. data.url)
end

rex.print.success("HTTP request complete")
