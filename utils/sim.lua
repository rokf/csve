
local data = {
  {"h1","h2","h3","h4"},
  {1,2,3,4},
  {1,2,3,4},
}

local tsize = {200,100}
local header_size = 3
local footer_size = 3
local side_margin = 1
local cell_width = 12
local rect_width
local rect_height = if (#data * 3) -1 < tsize[2] then

end
local rect = {0,0,1,1}
local position = {0,0}

if position[1] < rect[1] or position[2] < rect[2] then
  rect[1] = position[1]
  rect[2] = position[2]
elseif position[1] > (rect[1]+rect[3]) or position[2] > (rect[2] + rect[4]) then

end
