local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local EXIT_USAGE = 2
local UNKNOWN = fixtures.refusal("error-unknown-subcommand.json", EXIT_USAGE)

local function answering(fake, labels, paths)
  fake.reply("filter", fixtures.answer("filter-list.json"))
  fake.reply("category", fixtures.answer("category-list.json"))
  fake.reply("remote", fixtures.answer("remote-list.json"))
  fake.reply("label", labels or UNKNOWN)
  fake.reply("path", paths or UNKNOWN)
end

local function json(document)
  return { code = 0, stdout = vim.json.encode(document), stderr = "" }
end

local function read(app, cwd)
  local words = app.vocab:words(cwd)
  buffer.wait_for(function()
    return words.filters and words.categories and words.remotes and words.labels and words.paths
  end)
  return words
end

return {
  ["a folder's words come from dam's filter, category, remote, label and path lists, run there"] = function()
    local app, fake = helper.new()
    answering(fake)
    local words = read(app, "/tmp")
    local sent = vim.tbl_map(function(call)
      return table.concat(call.argv, " ") .. " in " .. call.cwd
    end, vim.list_slice(fake.calls, 2))
    table.sort(sent)
    assert(
      vim.deep_equal(sent, {
        "dam category list -o json in /tmp",
        "dam filter list -o json in /tmp",
        "dam label list -o json in /tmp",
        "dam path list -o json in /tmp",
        "dam remote list -o json in /tmp",
      }),
      vim.inspect(sent)
    )
    assert(vim.deep_equal(words.filters, { { word = "today", menu = "saved filter" } }), vim.inspect(words.filters))
    assert(vim.deep_equal(words.remotes, { { word = "fake", menu = "remote" } }), vim.inspect(words.remotes))
    local effort = words.categories[2]
    assert(effort.name == "effort" and vim.deep_equal(effort.values, { "light", "deep" }), vim.inspect(effort))
  end,

  ["labels and paths come with their counts when dam lists them"] = function()
    local app, fake = helper.new()
    answering(
      fake,
      json({ labels = { { name = "errand", count = 3 } } }),
      json({ paths = { { path = "home/", count = 1 } } })
    )
    local words = read(app, "/tmp")
    assert(vim.deep_equal(words.labels, { { word = "errand", menu = "label · 3" } }), vim.inspect(words.labels))
    assert(vim.deep_equal(words.paths, { { word = "home/", menu = "path · 1" } }), vim.inspect(words.paths))
  end,

  ["a dam without label or path lists gives none of them and says nothing"] = function()
    local app, fake = helper.new()
    answering(fake)
    local notes = buffer.capture_notes()
    local words = read(app, "/tmp")
    assert(vim.deep_equal(words.labels, {}) and vim.deep_equal(words.paths, {}), vim.inspect(words))
    assert(#notes == 0, vim.inspect(notes))
  end,

  ["any other refusal is said and leaves that list empty"] = function()
    local app, fake = helper.new()
    answering(fake)
    fake.reply("remote", fixtures.refusal("error-refused.json", 4))
    local notes = buffer.capture_notes()
    local words = read(app, "/tmp")
    assert(vim.deep_equal(words.remotes, {}), vim.inspect(words.remotes))
    assert(#notes == 1 and notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
  end,

  ["a folder is read once until DamChanged drops what was read"] = function()
    local app, fake = helper.new()
    answering(fake)
    read(app, "/tmp")
    local after_first = #fake.calls
    app.vocab:words("/tmp")
    vim.wait(30)
    assert(#fake.calls == after_first, vim.inspect(fake.calls))
    app.say:changed("/tmp")
    read(app, "/tmp")
    assert(#fake.calls == after_first + 5, #fake.calls)
  end,

  ["flags come from flags.json, per command"] = function()
    local app = helper.new()
    local due = vim.tbl_filter(function(flag)
      return flag.name == "--due"
    end, app.vocab:flags("new"))[1]
    assert(due and due.value == true and due.help:find("due date"), vim.inspect(due))
    assert(#app.vocab:flags("filter create") > 0)
  end,
}
