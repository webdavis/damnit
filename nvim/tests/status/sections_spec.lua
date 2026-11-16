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

  ["a change row shows what happened, the subject and the path"] = function()
    local lines = sections.render(doc({ unstaged = { change("update", "pay rent", "home/", { "due", "priority" }) } }))
    assert(lines[2] == "  update pay rent  home/  [due, priority]", vim.inspect(lines))
  end,

  ["a change at the root shows no path"] = function()
    local lines = sections.render(fixtures.decoded("status.json"))
    assert(vim.tbl_contains(lines, "  create water the plants  [subject]"), vim.inspect(lines))
  end,

  ["a waiting item shows what it is and who sent it"] = function()
    local lines = sections.render(fixtures.decoded("status.json"))
    assert(vim.tbl_contains(lines, "  new call the plumber  from tool"), vim.inspect(lines))
  end,

  ["a waiting set shows as one row with its count"] = function()
    local lines, _, targets = sections.render(fixtures.decoded("status.json"))
    local row = vim.fn.index(lines, "  set of 2  from tool") + 1
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
        return mark.group == "DamSection"
      end, marks)
    )
    assert(vim.deep_equal(marked, headers(lines)), vim.inspect(marked))
  end,

  ["each row names what a key acts on"] = function()
    local lines, _, targets = sections.render(fixtures.decoded("status.json"))
    local function at(text)
      return targets[vim.fn.index(lines, text) + 1]
    end
    local unstaged = at("  update call mom back  [subject]")
    assert(unstaged.kind == "change" and unstaged.section == "unstaged" and unstaged.id == ("2"):rep(40))
    local item = at("  new call the plumber  from tool")
    assert(item.kind == "waiting" and item.id == "<id-2>" and item.oid == "<id-2>", vim.inspect(item))
    assert(vim.deep_equal(at("Staged (1)"), { kind = "section", section = "staged" }), vim.inspect(at("Staged (1)")))
    assert(targets[vim.fn.index(lines, "") + 1] == nil)
  end,
}
