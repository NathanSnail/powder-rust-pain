local math = require("math")

function gen_p(index)
	return {
		colour = {math.random(), math.random(), math.random()},
		id = index,
		pos = { (index / 64 % 0.978241), (index / 50 % 0.832) }
	}
end

local out = {}

for i = 1,64*8 do
	table.insert(out,gen_p(i))
end

return out