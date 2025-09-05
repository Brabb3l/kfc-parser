if not loader.features.export then
	return
end

local function get_translations()
	---@type keen.LocaTagCollectionResource
	local localization = game.assets.get_resources_by_type("keen::LocaTagCollectionResource")[1].data
	local keenglishDataGuid = game.guid.from_content_hash(localization.keenglishDataHash)
	local buf = game.assets.get_content(keenglishDataGuid):read_data()

	---@type keen.LocaTagCollectionResourceData
	local localization_data = buf:read_resource("keen::LocaTagCollectionResourceData")

	local dict = {}

	for _, tag in ipairs(localization_data.tags) do
		dict[tag.id.value] = tag.text
	end

	return dict
end

local translations = get_translations()

---@type keen.ItemRegistryResource
local ItemRegistry = game.assets.get_resources_by_type("keen::ItemRegistryResource")[1].data
local item_list = "id,guid,name\n"

for _, itemRef in ipairs(ItemRegistry.itemRefs) do
	local item = game.assets.get_resource(itemRef, "keen::ItemInfo")

	if not item then
		warn("item not found for ref " .. itemRef)
		goto continue
	end

	--- @type keen.ItemInfo
	local data = item.data

	local id = data.itemId.value
	local guid = data.objectId
	local name = translations[game.guid.hash(data.name)] or data.debugName

	item_list = item_list .. (id .. "," .. guid .. "," .. name .. "\n")

	::continue::
end

io.export("item-list.csv", item_list)
