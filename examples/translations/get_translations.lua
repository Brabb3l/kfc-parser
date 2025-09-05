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

for key, text in pairs(get_translations()) do
	print(key, text)
end
