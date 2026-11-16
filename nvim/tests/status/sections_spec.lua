local fixtures = require("helpers.fixtures")
local sections = require("damnit.status.sections")

local EMPTY =
  { conflicts = {}, waiting = {}, sets = {}, unstaged = {}, staged = {}, unpushed = {}, notices = {}, warnings = {} }

local function doc(fields)
  return vim.tbl_extend("force", EMPTY, fields)
end

local function change(op, subject, path, fields)
  return { oid = ("a"):rep(40), op = op, subject = subject, path = path, fields = fields, kind = "task" }
end

local function right_of(marks, line)
  for _, mark in ipairs(marks) do
    if mark.line == line - 1 and mark.virt then
      return table.concat(vim.tbl_map(function(chunk)
        return chunk[1]
      end, mark.virt))
    end
  end
end

local function headers(lines)
  return vim.tbl_filter(function(line)
    return line ~= "" and not vim.startswith(line, " ")
  end, lines)
end

return {
  ["each section shows its count and empty ones are hidden"] = function()
    local lines = sections.render(fixtures.decoded("status.json"))
    local expected = { "Conflicts (1)", "Waiting (2)", "Unstaged (1)", "Staged (1)", "Unpushed (1)" }
    assert(vim.deep_equal(headers(lines), expected), vim.inspect(lines))
  end,

  ["an empty status says nothing is waiting or changed"] = function()
    local lines = sections.render(fixtures.decoded("status-empty.json"))
    assert(vim.deep_equal(lines, { "nothing changed, staged, waiting or unpushed" }), vim.inspect(lines))
  end,

  ["a change row shows the op and subject, with the path and changed fields at the right"] = function()
    local unstaged = { change("update", "pay rent", "home/", { "due", "priority" }), change("delete", "x", "", {}) }
    local lines, marks = sections.render(doc({ unstaged = unstaged }), 80)
    assert(lines[2] == "  update  pay rent", vim.inspect(lines))
    assert(right_of(marks, 2) == "  home/  due, priority", right_of(marks, 2))
    assert(right_of(marks, 3) == nil, vim.inspect(right_of(marks, 3)))
  end,

  ["a created task shows no field list"] = function()
    local lines, marks = sections.render(doc({ staged = { change("create", "plant", "home/", { "subject" }) } }), 80)
    assert(lines[2] == "  create  plant" and right_of(marks, 2) == "  home/", vim.inspect(right_of(marks, 2)))
  end,

  ["the op and subject line up in columns across sections"] = function()
    local lines = sections.render(fixtures.decoded("status.json"), 80)
    assert(vim.tbl_contains(lines, "  update  call mom back"), vim.inspect(lines))
    assert(vim.tbl_contains(lines, "  create  water the plants"), vim.inspect(lines))
  end,

  ["a long subject ends in an ellipsis before the right-hand columns"] = function()
    local long = change("update", ("a long subject "):rep(6), "home/", { "due" })
    local _, marks = sections.render(doc({ unstaged = { long } }), 60)
    assert(right_of(marks, 2) == "… home/  due", vim.inspect(right_of(marks, 2)))
  end,

  ["a waiting item shows what it is, the subject, who sent it and its age"] = function()
    local lines, marks = sections.render(fixtures.decoded("status.json"), 80)
    local row = vim.fn.index(lines, "  new       call the plumber") + 1
    assert(row > 0, vim.inspect(lines))
    assert(right_of(marks, row) == "  from tool  0s ago", vim.inspect(right_of(marks, row)))
  end,

  ["a waiting set shows as one row with its count"] = function()
    local lines, _, targets = sections.render(fixtures.decoded("status.json"), 80)
    local row = vim.fn.index(lines, "  set of 2") + 1
    assert(row > 0, vim.inspect(lines))
    assert(targets[row].kind == "set" and targets[row].id == "<id-3>", vim.inspect(targets[row]))
  end,

  ["unpushed shows each remote with its commit count"] = function()
    local unpushed =
      { { remote = "fake", commits = 2 }, { remote = "cal", commits = 1 }, { remote = "idle", commits = 0 } }
    local lines = sections.render(doc({ unpushed = unpushed }))
    assert(vim.deep_equal(lines, { "Unpushed (2)", "  fake  2 commits", "  cal  1 commit" }), vim.inspect(lines))
  end,

  ["a conflict row says to run dam resolve"] = function()
    local lines, _, targets = sections.render(fixtures.decoded("status.json"))
    local expected = "  buy soy milk  fake  run dam resolve 1111111 --ours or --theirs in a terminal"
    assert(lines[2] == expected, vim.inspect(lines))
    assert(targets[2].oid == ("1"):rep(40), vim.inspect(targets[2]))
  end,

  ["notices and warnings show under their own headers"] = function()
    local notices = { { kind = "missing_upstream", remote = "fake", tasks = {} } }
    local lines = sections.render(doc({ notices = notices, warnings = { "config.lua is readable by others" } }))
    local expected =
      { "Notices (1)", "  fake: missing upstream", "", "Warnings (1)", "  config.lua is readable by others" }
    assert(vim.deep_equal(lines, expected), vim.inspect(lines))
  end,

  ["each header is marked as a section"] = function()
    local lines, marks = sections.render(fixtures.decoded("status.json"))
    local marked = vim.tbl_map(
      function(mark)
        return lines[mark.line + 1]
      end,
      vim.tbl_filter(function(mark)
        return mark.group == "DamHeader"
      end, marks)
    )
    assert(vim.deep_equal(marked, headers(lines)), vim.inspect(marked))
  end,

  ["each row names what a key acts on"] = function()
    local lines, _, targets = sections.render(fixtures.decoded("status.json"))
    local function at(text)
      return targets[vim.fn.index(lines, text) + 1]
    end
    local unstaged = at("  update  call mom back")
    assert(unstaged.kind == "change" and unstaged.section == "unstaged" and unstaged.id == ("2"):rep(40))
    local item = at("  new       call the plumber")
    assert(item.kind == "waiting" and item.id == "<id-2>" and item.oid == "<id-2>", vim.inspect(item))
    assert(vim.deep_equal(at("Staged (1)"), { kind = "section", section = "staged" }), vim.inspect(at("Staged (1)")))
    assert(targets[vim.fn.index(lines, "") + 1] == nil)
  end,

  ["a change row takes the colour of its op"] = function()
    local unstaged = {
      change("create", "plant", "", { "subject" }),
      change("update", "water", "", { "due" }),
      change("delete", "weed", "", {}),
    }
    local lines, marks = sections.render(doc({ unstaged = unstaged }))
    local coloured = {}
    for _, mark in ipairs(marks) do
      if mark.group and mark.group ~= "DamHeader" then
        table.insert(coloured, lines[mark.line + 1]:sub(mark.col + 1, mark.end_col) .. " " .. mark.group)
      end
    end
    local expected = {
      "create  plant DamAdded",
      "update  water DamChanged",
      "delete  weed DamRemoved",
    }
    assert(vim.deep_equal(coloured, expected), vim.inspect(coloured))
  end,

  ["a narrow window drops the field list first, then the path, so sixteen cells of subject still show"] = function()
    local unstaged = { change("update", "pay rent", "home/", { "due", "priority" }) }
    local _, marks = sections.render(doc({ unstaged = unstaged }), 40)
    assert(right_of(marks, 2) == "  home/", vim.inspect(right_of(marks, 2)))
    local _, narrower = sections.render(doc({ unstaged = unstaged }), 14)
    assert(right_of(narrower, 2) == "…", vim.inspect(right_of(narrower, 2)))
  end,

  ["age reads in the largest whole unit"] = function()
    local columns = require("damnit.status.columns")
    local ages = vim.tbl_map(columns.ago, { 0, 59, 60, 3599, 3600, 86399, 86400 * 3 })
    local expected = { "0s ago", "59s ago", "1m ago", "59m ago", "1h ago", "23h ago", "3d ago" }
    assert(vim.deep_equal(ages, expected), vim.inspect(ages))
  end,
}
