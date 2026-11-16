local fixtures = require("helpers.fixtures")
local groups = require("damnit.list.groups")

local function titles(segments)
  return vim.tbl_map(function(segment)
    return segment.title .. " " .. #segment.objects
  end, segments)
end

local function task(fields)
  return vim.tbl_extend("keep", fields, { kind = "task", path = "", labels = {}, task = { priority = 4 } })
end

return {
  ["a view sorted by path gets a group wherever the path changes, and none for the empty path"] = function()
    local objects = fixtures.decoded("list-by-path.json").objects
    local got = titles(groups.segments(objects, "path"))
    assert(vim.deep_equal(got, { "none 1", "home/ 2", "work/ 2" }), vim.inspect(got))
  end,

  ["only the first sort key groups, and :desc is not part of its name"] = function()
    local objects = fixtures.decoded("list-by-path.json").objects
    local got = titles(groups.segments(objects, { "path:desc", "due" }))
    assert(vim.deep_equal(got, { "none 1", "home/ 2", "work/ 2" }), vim.inspect(got))
    got = titles(groups.segments(objects, "path,due"))
    assert(vim.deep_equal(got, { "none 1", "home/ 2", "work/ 2" }), vim.inspect(got))
  end,

  ["a date key groups by the first ten characters of dam's text"] = function()
    local objects = {
      task({ task = { due = "2026-11-20T09:00:00-05:00" } }),
      task({ task = { due = "2026-11-20" } }),
      task({ task = { due = "2026-11-21T10:00" } }),
      task({}),
    }
    local got = titles(groups.segments(objects, "due"))
    assert(vim.deep_equal(got, { "2026-11-20 2", "2026-11-21 1", "none 1" }), vim.inspect(got))
  end,

  ["any other key groups by the value as dam printed it"] = function()
    local objects = {
      task({ task = { priority = 1 } }),
      task({ task = { priority = 1 } }),
      task({ task = { priority = 4 } }),
      task({ subject = "x", author = "owner" }),
    }
    local got = titles(groups.segments(objects, "priority"))
    assert(vim.deep_equal(got, { "1 2", "4 2" }), vim.inspect(got))
    got = titles(groups.segments({ task({ labels = { "a", "b" } }), task({}) }, "labels"))
    assert(vim.deep_equal(got, { "a, b 1", "none 1" }), vim.inspect(got))
  end,

  ["with no sort there are no groups"] = function()
    assert(groups.segments(fixtures.decoded("list-by-path.json").objects, nil) == nil)
  end,
}
