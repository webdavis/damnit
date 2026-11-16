local fixtures = require("helpers.fixtures")
local rows = require("damnit.log.rows")

local function commits()
  return fixtures.decoded("log.json").commits
end

local function texts(virt_lines)
  return vim.tbl_map(function(chunks)
    return table.concat(vim.tbl_map(function(chunk)
      return chunk[1]
    end, chunks))
  end, virt_lines)
end

return {
  ["each commit is one row of aligned columns: short hash, time, message, change count, subjects"] = function()
    local lines = rows.render(commits(), 200)
    local expected = {
      "<id-1>  <now>  call back        1 change   call mom back",
      "<id-2>  <now>  plan the errand  1 change   buy oat milk",
      "<id-3>  <now>  seed the week    4 changes  buy oat milk, call mom, standup, offsite",
    }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["the short hash is seven characters"] = function()
    local commit = vim.tbl_extend("force", commits()[1], { id = ("a"):rep(40) })
    local lines = rows.render({ commit }, 200)
    assert(vim.startswith(lines[1], ("a"):rep(7) .. "  "), lines[1])
  end,

  ["the time is dam's own text cut to the minute"] = function()
    local commit = vim.tbl_extend("force", commits()[1], { at = "2026-11-16T15:59:13.851186Z" })
    local lines = rows.render({ commit }, 200)
    assert(lines[1]:find("  2026-11-16 15:59Z  ", 1, true), lines[1])
  end,

  ["a row wider than the window ends in an ellipsis at its edge"] = function()
    local _, marks = rows.render(commits(), 60)
    local cut = vim.tbl_filter(function(mark)
      return mark.virt and mark.virt[1][1] == "…"
    end, marks)
    assert(#cut == 1 and cut[1].line == 2 and cut[1].align == "right_align", vim.inspect(marks))
  end,

  ["the hash, time and count are dimmed"] = function()
    local _, marks = rows.render({ commits()[1] }, 200)
    local dimmed = vim.tbl_map(function(mark)
      return { mark.col, mark.end_col, mark.group }
    end, marks)
    local expected = { { 0, 6, "DamMeta" }, { 8, 13, "DamMeta" }, { 26, 34, "DamMeta" } }
    assert(vim.deep_equal(dimmed, expected), vim.inspect(dimmed))
  end,

  ["a commit's changes are lines of op, subject and fields, the op coloured"] = function()
    local shown = rows.changes(fixtures.decoded("show-commit.json"))
    assert(vim.deep_equal(texts(shown), { "    update  buy oat milk  labels, priority, due" }), vim.inspect(shown))
    assert(shown[1][2][2] == "DamChanged", vim.inspect(shown))
  end,
}
