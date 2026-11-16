local fixtures = require("helpers.fixtures")
local winbar = require("damnit.list.winbar")

local function text_of(chunks)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, chunks))
end

return {
  ["the winbar names the view and the project, then counts rows and what dam marked"] = function()
    local objects = fixtures.decoded("list-tree.json").objects
    local chunks = winbar.chunks("today", "work/", objects)
    assert(text_of(chunks) == " today · work/ · 6 tasks · 2 overdue · 1 blocked", text_of(chunks))
  end,

  ["counts of nothing are left out and one is singular"] = function()
    local objects = fixtures.decoded("list.json").objects
    local chunks = winbar.chunks("default", "app", { objects[1] })
    assert(text_of(chunks) == " default · app · 1 task", text_of(chunks))
    assert(text_of(winbar.chunks("default", "app", {})) == " default · app · 0 tasks")
  end,

  ["events are counted apart from tasks"] = function()
    local event = fixtures.decoded("show-event.json")
    local tasks = fixtures.decoded("list.json").objects
    local chunks = winbar.chunks("week", "app", { tasks[1], event, event })
    assert(text_of(chunks) == " week · app · 1 task · 2 events", text_of(chunks))
  end,

  ["the view is a header and the overdue and blocked counts take their own colour"] = function()
    local chunks = winbar.chunks("today", "work/", fixtures.decoded("list-tree.json").objects)
    local groups = {}
    for _, chunk in ipairs(chunks) do
      if chunk[2] then
        groups[vim.trim(chunk[1])] = chunk[2]
      end
    end
    local expected = { today = "DamHeader", ["2 overdue"] = "DamOverdue", ["1 blocked"] = "DamBlocked" }
    assert(vim.deep_equal(groups, expected), vim.inspect(chunks))
  end,

  ["the winbar is cut after the view and the project, so the counts go first"] = function()
    local chunks = winbar.chunks("today", "work/", {})
    assert(vim.deep_equal(chunks[4], { "", cut = true }), vim.inspect(chunks))
  end,
}
