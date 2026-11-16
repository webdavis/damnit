local M = {}

---@class damnit.Hosts
---@field rex damnit.Host
---@field clipboard damnit.Host

---@param app damnit.App
---@return damnit.Hosts
function M.made(app)
  return { rex = require("damnit.handoff.rex").new(app.process), clipboard = require("damnit.handoff.clipboard").new() }
end

---@param want "auto"|"rex"|"clipboard"
---@param hosts damnit.Hosts
---@return damnit.Host host, string|nil unavailable why the chosen host cannot take the brief
function M.chosen(want, hosts)
  if want == "clipboard" then
    return hosts.clipboard
  end
  local available, why = hosts.rex.available()
  if available then
    return hosts.rex
  end
  if want == "rex" then
    return hosts.rex, why
  end
  return hosts.clipboard
end

return M
