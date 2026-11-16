local hosts = require("damnit.handoff.hosts")
local report = require("damnit.health.report")

local M = {}

---@param item damnit.HealthItem
local function shown(item)
  vim.health[item.level](item.text, item.advice)
end

---@param app damnit.App
---@return damnit.Doctor|nil doc, damnit.Error|nil err
local function doctor(app)
  local got
  app.dam:run({ args = { "doctor" }, cwd = vim.fn.getcwd(), key = "health", report = true }, function(doc, err)
    got = { doc = doc, err = err }
  end)
  vim.wait(app.options.timeout_ms, function()
    return got ~= nil
  end, 10)
  return got and got.doc, got and got.err
end

---@param app damnit.App
local function host(app)
  local made = hosts.made(app)
  local want = app.options.handoff.host
  local chosen, unavailable = hosts.chosen(want, made)
  local _, rex_why = made.rex.available()
  vim.health.start("hand-off")
  shown(report.host(want, chosen.name, unavailable, rex_why))
end

---@param app? damnit.App
function M.check(app)
  app = app or require("damnit").app()
  local doc, err = doctor(app)
  vim.health.start("dam")
  local found = app.dam:version()
  shown(report.dam(found, not found and err or nil))
  if doc then
    vim.health.start("dam doctor")
    vim.tbl_map(shown, report.checks(doc))
  elseif found and err then
    shown({ level = "error", text = "dam doctor failed: " .. err.message })
  end
  host(app)
end

return M
