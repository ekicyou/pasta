--collectgarbage("stop")

local i = 0
while i<1000000 do
  local temp = {{}}
  i = i + 1
end

--collectgarbage("restart")

collectgarbage("collect")
