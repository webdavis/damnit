local M = { MIN = "0.2.0", BELOW = "0.3.0" }

local function parts(text)
  local major, minor, patch = tostring(text):match("^(%d+)%.(%d+)%.(%d+)$")
  if major then
    return { tonumber(major), tonumber(minor), tonumber(patch) }
  end
end

local function compare(left, right)
  for i = 1, 3 do
    if left[i] ~= right[i] then
      return left[i] < right[i] and -1 or 1
    end
  end
  return 0
end

---@param banner string
---@return string|nil
function M.of(banner)
  return tostring(banner):match("^%s*dam%s+(%d+%.%d+%.%d+)")
end

---@param version string
---@return boolean
function M.supported(version)
  local given = parts(version)
  return given ~= nil and compare(given, parts(M.MIN)) >= 0 and compare(given, parts(M.BELOW)) < 0
end

---@param version string
---@return string
function M.refusal(version)
  return ("dam %s is outside >=%s <%s; update dam or the plugin"):format(version, M.MIN, M.BELOW)
end

return M
