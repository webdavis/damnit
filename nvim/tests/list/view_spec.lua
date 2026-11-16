local view = require("damnit.list.view")

local function call(name, views)
  local args, stdin = view.call(name, views or {})
  return { args = args, stdin = stdin and vim.json.decode(stdin) }
end

return {
  ["the default view runs plain dam list"] = function()
    assert(vim.deep_equal(call(nil), { args = { "list" } }), vim.inspect(call(nil)))
  end,

  ["a view name not in options is sent as --filter"] = function()
    local got = call("today")
    assert(vim.deep_equal(got, { args = { "list", "--filter", "today" } }), vim.inspect(got))
  end,

  ["a view name naming a saved filter is sent as --filter with that filter"] = function()
    local got = call("now", { now = "today" })
    assert(vim.deep_equal(got, { args = { "list", "--filter", "today" } }), vim.inspect(got))
  end,

  ["a view table is sent as --request on stdin"] = function()
    local got = call("open", { open = { done = "any", labels = { "a", "b" } } })
    local expected = { args = { "list", "--request", "-" }, stdin = { done = "any", labels = { "a", "b" } } }
    assert(vim.deep_equal(got, expected), vim.inspect(got))
  end,

  ["a global view sends global in its request"] = function()
    local got = call("everything", { everything = { global = true } })
    local expected = { args = { "list", "--request", "-" }, stdin = { global = true } }
    assert(vim.deep_equal(got, expected), vim.inspect(got))
  end,

  ["a view given as a table is sent as it is"] = function()
    local got = call({ kind = "event" })
    assert(vim.deep_equal(got.stdin, { kind = "event" }), vim.inspect(got))
  end,

  ["a view's single path is the one folder it lists"] = function()
    local views = { work = { path = "work/" }, both = { path = "a/,b/" }, listed = { path = { "home/" } } }
    assert(view.single_path("work", views) == "work/")
    assert(view.single_path("listed", views) == "home/")
    assert(view.single_path("both", views) == nil)
    assert(view.single_path("today", views) == nil)
    assert(view.single_path(nil, views) == nil)
    assert(view.single_path({ path = { "a/", "b/" } }, views) == nil)
  end,
}
