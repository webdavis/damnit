local words = require("damnit.kernel.words")

local FLAGS = {
  { name = "--due", value = "required", help = "Set the due date" },
  { name = "--deadline", value = "required", help = "Set the deadline" },
  { name = "--labels", value = "required", help = "Add labels" },
  { name = "--category", value = "required", help = "Give it LABEL of GROUP" },
  { name = "--path", value = "required", help = "Put it under PATH" },
  { name = "--filter", value = "required", help = "Act on a saved filter" },
  { name = "--preview", value = "absent", help = "Show the change" },
}

local VOCAB = {
  labels = { { word = "errand", menu = "3 tasks" }, { word = "home", menu = "1 task" } },
  paths = { { word = "work/", menu = "5 tasks" }, { word = "work/clients/", menu = "2 tasks" } },
  filters = { { word = "today", menu = "saved filter" } },
  remotes = {},
  categories = { { name = "effort", values = { "deep", "light" } }, { name = "energy", values = { "high" } } },
}

local function at(line)
  return words.at(line, #line, FLAGS, VOCAB)
end

local function said(start, candidates, want_start, want)
  assert(start == want_start, ("start %d, not %d"):format(start, want_start))
  assert(vim.deep_equal(candidates, want), vim.inspect(candidates))
end

local function names(candidates)
  return vim.tbl_map(function(candidate)
    return candidate.word
  end, candidates)
end

return {
  ["a word starting with - completes the flag names it begins, each with its help line"] = function()
    local start, candidates = at("buy milk --d")
    said(start, candidates, 9, {
      { word = "--due", menu = "Set the due date" },
      { word = "--deadline", menu = "Set the deadline" },
    })
  end,

  ["an empty word outside a value offers every flag"] = function()
    local start, candidates = at("buy milk ")
    assert(start == 9 and #candidates == #FLAGS, vim.inspect(candidates))
  end,

  ["a subject word completes nothing"] = function()
    local start, candidates = at("buy mi")
    said(start, candidates, 4, {})
  end,

  ["the word after --labels completes dam's labels with their note"] = function()
    local start, candidates = at("buy --labels er")
    said(start, candidates, 13, { { word = "errand", menu = "3 tasks" } })
  end,

  ["a comma list completes its last item"] = function()
    local start, candidates = at("buy --labels home,er")
    said(start, candidates, 18, { { word = "errand", menu = "3 tasks" } })
  end,

  ["the word after --path completes dam's paths, and after --filter its saved filters"] = function()
    local _, paths = at("buy --path work/c")
    assert(vim.deep_equal(names(paths), { "work/clients/" }), vim.inspect(paths))
    local _, filters = at("--filter ")
    assert(vim.deep_equal(names(filters), { "today" }), vim.inspect(filters))
  end,

  ["--category completes GROUP= first"] = function()
    local start, candidates = at("buy --category e")
    said(start, candidates, 15, {
      { word = "effort=", menu = "deep, light" },
      { word = "energy=", menu = "high" },
    })
  end,

  ["--category GROUP= completes that group's values and none"] = function()
    local start, candidates = at("buy --category effort=")
    said(start, candidates, 15, {
      { word = "effort=deep", menu = "effort" },
      { word = "effort=light", menu = "effort" },
      { word = "effort=none", menu = "takes effort's label off" },
    })
    local _, narrowed = at("buy --category effort=l")
    assert(vim.deep_equal(names(narrowed), { "effort=light" }), vim.inspect(narrowed))
  end,

  ["a value dam gives no words for completes nothing"] = function()
    local start, candidates = at("buy --due fr")
    said(start, candidates, 10, {})
  end,

  ["the word after a flag that takes no value is not its value"] = function()
    local _, candidates = at("buy --preview ")
    assert(#candidates == #FLAGS, vim.inspect(candidates))
  end,

  ["only the text before the cursor counts"] = function()
    local line = "buy --labels er --due fri"
    local start, candidates = words.at(line, #"buy --labels er", FLAGS, VOCAB)
    said(start, candidates, 13, { { word = "errand", menu = "3 tasks" } })
  end,

  ["a flag that names its values completes from them"] = function()
    local flags = { { name = "--overdue", value = "optional", values = { "true", "false", "any" }, help = "" } }
    local start, candidates = words.at("--overdue f", 11, flags, VOCAB)
    said(start, candidates, 10, { { word = "false", menu = "overdue" } })
  end,
}
