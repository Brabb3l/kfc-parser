---@param locale keen.LanguageId|nil the locale identifier. If nil, falls back to "keenglish"
---@return table<u32, string> dictionary of translations
local function get_translations(locale)
	---@type keen.LocaTagCollectionResource
	local localization = game.assets.get_resources_by_type("keen::LocaTagCollectionResource")[1].data
	local content_hash = nil

	if locale ~= nil then
		-- look for the specified locale
		for _, loc in ipairs(localization.languages) do
			if loc.language == locale then
				content_hash = loc.dataHash
				break
			end
		end
	else
		-- fall back to keenglish
		content_hash = localization.keenglishDataHash
	end

	if content_hash == nil then
		error("Locale not found: " .. tostring(locale))
	end

	-- load the localization data
	local guid = game.guid.from_content_hash(content_hash)
	local buf = game.assets.get_content(guid):read_data()

	---@type keen.LocaTagCollectionResourceData
	local localization_data = buf:read_resource("keen::LocaTagCollectionResourceData")

	-- build the dictionary
	local dict = {}

	for _, tag in ipairs(localization_data.tags) do
		dict[tag.id.value] = tag.text
	end

	return dict
end

for key, text in pairs(get_translations("En_Us")) do
	print(key, text)
end
