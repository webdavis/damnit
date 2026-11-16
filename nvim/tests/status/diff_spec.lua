local diff = require("damnit.status.diff")
local fixtures = require("helpers.fixtures")

local function texts(virt_lines)
  return vim.tbl_map(function(chunks)
    return table.concat(vim.tbl_map(function(chunk)
      return chunk[1]
    end, chunks))
  end, virt_lines)
end

local function status()
  return fixtures.decoded("status-full.json")
end

return {
  ["a change shows each field it changed with dam's value before and after"] = function()
    local lines = texts(diff.lines({ status().unstaged[1] }))
    local expected = {
      "    labels    none → phone",
      "    priority  4 → 1",
      "    due       none → 2026-09-30",
    }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["a list is joined with commas, as the confirm float shows it"] = function()
    local lines = texts(diff.lines({ status().staged[1] }))
    assert(lines[1] == "    labels    errand → errand,shop", vim.inspect(lines))
  end,

  ["a new item has nothing before"] = function()
    local lines = texts(diff.lines({ status().waiting[1] }))
    assert(vim.deep_equal(lines, { "    subject  none → call the plumber" }), vim.inspect(lines))
  end,

  ["a set shows each item's subject before its fields, aligned"] = function()
    local lines = texts(diff.lines(status().sets[1].items, true))
    local expected = {
      "    call mom      priority  1 → 3",
      "    buy oat milk  priority  4 → 3",
    }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["the field and arrow are dimmed and the values keep their colour"] = function()
    local chunks = diff.lines({ status().unstaged[1] })[2]
    local groups = vim.tbl_map(function(chunk)
      return chunk[2] or ""
    end, chunks)
    assert(vim.deep_equal(groups, { "DamMeta", "DamRemoved", "DamMeta", "DamAdded" }), vim.inspect(chunks))
  end,

  ["a value over several lines is drawn on one"] = function()
    local change = { fields = { "body" }, before = { body = "one\ntwo" }, after = { body = "" } }
    local lines = texts(diff.lines({ change }))
    assert(lines[1] == "    body  one two → none", vim.inspect(lines))
  end,

  ["a removal names what dam removes, in one line"] = function()
    local lines = diff.lines({ status().unstaged[2] })
    assert(vim.deep_equal(texts(lines), { "    delete: the event is removed" }), vim.inspect(lines))
    assert(lines[1][1][2] == "DamRemoved", vim.inspect(lines))
  end,

  ["a change with no fields and nothing removed has no lines"] = function()
    local lines = diff.lines({ { op = "update", kind = "task", fields = {} } })
    assert(vim.deep_equal(lines, {}), vim.inspect(lines))
  end,
}
