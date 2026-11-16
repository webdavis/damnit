local history = require("damnit.kernel.history")

return {
  ["older walks back from the newest entry and stops at the oldest"] = function()
    local browse = history.browse({ "one", "two" })
    assert(browse:older("draft") == "two")
    assert(browse:older("two") == "one")
    assert(browse:older("one") == nil)
  end,

  ["newer walks forward and ends at the text typed before browsing"] = function()
    local browse = history.browse({ "one", "two" })
    browse:older("draft")
    browse:older("two")
    assert(browse:newer() == "two")
    assert(browse:newer() == "draft")
    assert(browse:newer() == nil)
  end,

  ["an empty history gives nothing"] = function()
    local browse = history.browse({})
    assert(browse:older("draft") == nil and browse:newer() == nil)
  end,

  ["kept adds an entry unless it repeats the newest one"] = function()
    local entries = {}
    history.kept(entries, "one")
    history.kept(entries, "one")
    history.kept(entries, "two")
    assert(vim.deep_equal(entries, { "one", "two" }), vim.inspect(entries))
  end,
}
