local M = {}

---@return damnit.Options
function M.defaults()
  return {
    dam = "dam",
    views = {},
    default_view = nil,
    timeout_ms = 120000,
    sidebar = { side = "left", width = 40 },
    handoff = { host = "auto" },
    signs = { overdue = "!", blocked = "~", changed = "M", staged = "S", waiting = "?" },
  }
end

local KNOWN = {
  dam = true,
  views = true,
  default_view = true,
  timeout_ms = true,
  sidebar = { side = true, width = true },
  handoff = { host = true },
  signs = { overdue = true, blocked = true, changed = true, staged = true, waiting = true },
}

local function unknown_keys(given, known, prefix, found)
  for key, value in pairs(given) do
    local name = prefix .. tostring(key)
    if not known[key] then
      table.insert(found, name)
    elseif type(known[key]) == "table" and type(value) == "table" then
      unknown_keys(value, known[key], name .. ".", found)
    end
  end
  return found
end

local function one_of(...)
  local allowed = { ... }
  return function(value)
    return vim.tbl_contains(allowed, value), table.concat(allowed, " or ")
  end
end

local function sign_text(value)
  local width = type(value) == "string" and vim.fn.strdisplaywidth(value) or 0
  return width >= 1 and width <= 2, "text one or two cells wide"
end

local function validate(o)
  vim.validate("dam", o.dam, "string")
  vim.validate("views", o.views, "table")
  vim.validate("default_view", o.default_view, { "string", "table" }, true)
  vim.validate("timeout_ms", o.timeout_ms, "number")
  vim.validate("sidebar", o.sidebar, "table")
  vim.validate("sidebar.side", o.sidebar.side, one_of("left", "right"))
  vim.validate("sidebar.width", o.sidebar.width, "number")
  vim.validate("handoff", o.handoff, "table")
  vim.validate("handoff.host", o.handoff.host, one_of("auto", "rex", "clipboard"))
  vim.validate("signs", o.signs, "table")
  for name, text in pairs(o.signs) do
    vim.validate("signs." .. name, text, sign_text)
  end
  for name, view in pairs(o.views) do
    vim.validate("views." .. tostring(name), view, { "string", "table" })
  end
end

---@param user table
---@return damnit.Options
function M.resolve(user)
  vim.validate("options", user, "table")
  local unknown = unknown_keys(user, KNOWN, "", {})
  if #unknown > 0 then
    table.sort(unknown)
    error(("damnit: unknown option %s"):format(table.concat(unknown, ", ")), 0)
  end
  local options = vim.tbl_deep_extend("force", M.defaults(), user)
  validate(options)
  return options
end

return M
