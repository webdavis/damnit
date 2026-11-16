local buffer = require("helpers.buffer")
local clipboard = require("helpers.clipboard")
local real_dam = require("helpers.real_dam")

local function handed_off(sandbox)
  local options = require("damnit.kernel.options").resolve({ dam = real_dam.bin, handoff = { host = "clipboard" } })
  local app = require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
  local oid = real_dam.made(app, { "new", "plant the tomatoes", "--labels", "garden" }, sandbox)
  local held = clipboard.fake()
  vim.fn.setreg('"', "")
  require("damnit.handoff").new(app).hand_off(oid, sandbox, "start by the fence")
  buffer.wait_for(function()
    return vim.fn.getreg('"') ~= ""
  end, 20000)
  local brief = vim.fn.getreg('"')
  assert(table.concat(held["+"], "\n") == brief, vim.inspect(held))
  assert(brief:find("task: plant the tomatoes", 1, true), brief)
  assert(brief:find("oid: " .. oid:sub(1, 7), 1, true), brief)
  assert(brief:find("dam show " .. oid .. " -o json", 1, true), brief)
end

return {
  ["hand-off with the clipboard host puts a brief holding the task's subject and oid in the register"] = function()
    return real_dam.with_real_dam(handed_off)
  end,
}
