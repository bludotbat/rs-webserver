return function()
    if not request.body then
        result.data = "no prams passed"
        result.status = 500

        return
    end

    print("---API RESULT---")

    for k, v in pairs(request.body) do
        print(k .. ": " .. v)
    end

    print("---END RESULT---")
    result.status = 200
    result.data = "hello post request"
end