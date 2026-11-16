local options = require("damnit.kernel.options")

return {
  ["unknown options are refused by name"] = function()
    local ok, err = pcall(options.resolve, { timeout = 5, sidebar = { wdith = 30 } })
    assert(not ok, "unknown options were taken")
    assert(err:find("sidebar.wdith", 1, true) and err:find("timeout", 1, true), err)
  end,

  ["a known option keeps the defaults it does not name"] = function()
    local resolved = options.resolve({ sidebar = { width = 30 } })
    assert(resolved.sidebar.width == 30 and resolved.sidebar.side == "left" and resolved.dam == "dam")
  end,

  ["signs default to plain ASCII and a changed one keeps the rest"] = function()
    local signs = options.resolve({ signs = { overdue = "!!" } }).signs
    local expected = { overdue = "!!", blocked = "~", changed = "M", staged = "S", waiting = "?" }
    assert(vim.deep_equal(signs, expected), vim.inspect(signs))
  end,

  ["a sign wider than two cells or not text is refused by name"] = function()
    for _, given in ipairs({ { blocked = "WAIT" }, { blocked = 1 }, { blocked = "" } }) do
      local ok, err = pcall(options.resolve, { signs = given })
      assert(not ok and err:find("signs.blocked", 1, true), vim.inspect(given) .. " " .. tostring(err))
    end
    local ok, err = pcall(options.resolve, { signs = { late = "L" } })
    assert(not ok and err:find("signs.late", 1, true), tostring(err))
  end,
}
