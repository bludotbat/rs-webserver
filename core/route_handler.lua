local route = {}
route.routes = {}

function route:add(path, method, callback)
    table.insert(route.routes, {path, method, callback})
end

function route:addController(path, method, controller)
    table.insert(route.routes, {path, method, require("controller/" .. controller)})
end

function route:finish()
    for k, v in ipairs(route.routes) do
        if request.endpoint == v[1] and request.method == v[2] then
            v[3]()
            return
        end
    end

    result.status = 404
    result.data = "Requested resource was not found!"
end

return route