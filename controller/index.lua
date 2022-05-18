
local function CalculateSomeBigMaths(a, b, c)
    return a * b * c
end

local function Render()
    local renderView = require("core/view_render")

    result.status = 200
    result.data = renderView("index", {["fb"] = "from render view " .. CalculateSomeBigMaths(5, os.time(), 20), ["wasfun"] = true, ["render"] = renderView})
end

return Render