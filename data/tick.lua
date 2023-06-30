--[[
	Tech Demo Lua Program
	Demonstrates the capabilities of Powder RS
	By: Nathan
]]


-- demo fps display, just showing it can do everything rust can.

math = require("math")

st = st or
	GetTime() -- im not going to write a vscode language server for these funcs so their syntax wont get highlighted.
print("fps: " .. tostring(GetFrame() / (GetTime() - st) * 1000))

local data = EntityGetComponentValue(0, "sprite.pos")
EntitySetComponentValue(0, "sprite.pos", { data.x + 0.50, data.y })
local data = EntityGetComponentValue(0, "sprite.pos")
EntitySetComponentValue(0, "sprite.pos", { data.x - 0.50, data.y }) -- editing works multiple times / frame

local data = EntityGetComponentValue(0, "deleted")
print(data)                                       -- multiple types
EntitySetComponentValue(0, "deleted", { not data }) -- deleting entities (note handling undeletion is not garunteed - new entities could have overwritten data)
-- this also creates a strange visual effect depending on monitor due to ghosting taking a frame to clear.




if math.random() < 0.05 then
	CreateEntity() -- we can use this in 1 frame
end

-- RS_deltas = {{0,"data","fish"}} RS_deltas can be manually handled in here, if you are a bit crazy
