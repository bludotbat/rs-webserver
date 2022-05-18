return function(view_name, data)
    _G["data"] = data
    local f = io.open("views/" .. view_name .. ".view")
    local view_data = f:read("*all")
    f:close()
    local to_eval = {} -- basic lua template renderer that evalutes a lua string to a variable then renders it

    -- get a list of strings to evaluate and replace, do it like this so we dont need to do multiple evaluations for dupes
    for x in string.gmatch(view_data, "{{(.-)}}") do
        to_eval[x] = true
    end

    for k, v in pairs(to_eval) do
        load("cv = " .. k)()
        local pattern = ("{{" .. k .. "}}"):gsub("([" .. ("%^$().[]*+-?"):gsub("(.)", "%%%1") .. "])", "%%%1")

        if cv then
            view_data = view_data:gsub(pattern, cv)
        else
            view_data = view_data:gsub(pattern, "") -- if the expression does not return a valid value, replace it with nothing
        end

        cv = nil
    end

    return view_data
end