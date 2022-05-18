local route = require("core/route_handler")

route:addController("/", "GET", "index")
route:addController("/test", "POST", "testapi")

route:finish()