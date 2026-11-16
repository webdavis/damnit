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
}
