local buffer = require("helpers.buffer")
local list = require("helpers.list")

local EXIT_REFUSED = 4

local function refusing(oids)
  local err = { error = { kind = "refused", rule = "focus_full", message = "the day is full", oids = oids } }
  return { code = EXIT_REFUSED, stdout = "", stderr = vim.json.encode(err) }
end

local function lines_with_errors()
  buffer.wait_for(function()
    return #vim.diagnostic.get(0) > 0
  end)
  local lines = vim.tbl_map(function(diagnostic)
    return buffer.lines()[diagnostic.lnum + 1]
  end, vim.diagnostic.get(0))
  table.sort(lines)
  return lines
end

return {
  ["a refused change whose named oids are not the targets shows on the target row too"] = function()
    local _, fake = list.opened()
    fake.reply("done", refusing({ list.OAT_MILK }))
    list.on("[ ] call mom")
    buffer.feed("x")
    local lines = lines_with_errors()
    assert(vim.deep_equal(lines, { "[ ] call mom", "[ ] p2 buy oat milk" }), vim.inspect(lines))
  end,

  ["a refused change that names its target shows only on the rows dam names"] = function()
    local _, fake = list.opened()
    fake.reply("done", refusing({ list.OAT_MILK }))
    list.on("[ ] p2 buy oat milk")
    buffer.feed("x")
    local lines = lines_with_errors()
    assert(vim.deep_equal(lines, { "[ ] p2 buy oat milk" }), vim.inspect(lines))
  end,
}
