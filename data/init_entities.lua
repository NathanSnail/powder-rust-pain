o = { {
	Hitbox = {
		size = { 0.2, 1.0 },
		pos = { 0.3, 0.8 },
		simulate = true,
		mass = 1,
	},
	Sprite = {
		size = {0.25, 0.25},
		scale = {3.0,3.0},
	}
} }
for i = 1, 32 do
	table.insert(o, { deleted = true, Hitbox = { deleted = true }, Sprite = { deleted = true } }) -- allocate memory for more entities
	-- in this toy example itll crash pretty fast because we allocate way too little memory, just showing POC
end
return o
