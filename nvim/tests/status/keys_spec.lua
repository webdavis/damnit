local buffer = require("helpers.buffer")
local status = require("helpers.status")
local ui = require("helpers.ui")

local MOM = ("2"):rep(40)
local UNSTAGED = "  update call mom back  [subject]"
local STAGED = "  create water the plants  [subject]"
local ITEM = "  new call the plumber  from tool"
local SET = "  set of 2  from tool"

return {
  ["s on a change stages it"] = function()
    local _, fake = status.opened()
    status.on(UNSTAGED)
    buffer.feed("s")
    assert(vim.deep_equal(status.sent(fake, "add"), { "add", MOM }), vim.inspect(status.sent(fake, "add")))
  end,

  ["s on a section header stages all"] = function()
    local _, fake = status.opened()
    status.on("Unstaged (1)")
    buffer.feed("s")
    assert(vim.deep_equal(status.sent(fake, "add"), { "add", "-A" }), vim.inspect(status.sent(fake, "add")))
  end,

  ["u unstages"] = function()
    local _, fake = status.opened()
    status.on(STAGED)
    buffer.feed("u")
    assert(vim.deep_equal(status.sent(fake, "reset"), { "reset", "<id-1>" }), vim.inspect(status.sent(fake, "reset")))
  end,

  ["a on a waiting item accepts it"] = function()
    local _, fake = status.opened()
    status.on(ITEM)
    buffer.feed("a")
    assert(vim.deep_equal(status.sent(fake, "add"), { "add", "<id-2>" }), vim.inspect(status.sent(fake, "add")))
  end,

  ["x on a waiting item declines it only after a yes"] = function()
    local _, fake = status.opened()
    status.on(ITEM)
    ui.answering({ select = "keep" }, function(prompts)
      buffer.feed("x")
      assert(prompts[1] == "decline new call the plumber  from tool?", vim.inspect(prompts))
    end)
    status.nothing_sent(fake, "rm")
    ui.answering({ select = "decline" }, function()
      buffer.feed("x")
    end)
    assert(vim.deep_equal(status.sent(fake, "rm"), { "rm", "<id-2>" }), vim.inspect(status.sent(fake, "rm")))
  end,

  ["x on a set declines the set after a yes"] = function()
    local _, fake = status.opened()
    status.on(SET)
    ui.answering({ select = "decline" }, function()
      buffer.feed("x")
    end)
    assert(vim.deep_equal(status.sent(fake, "rm"), { "rm", "<id-3>" }), vim.inspect(status.sent(fake, "rm")))
  end,

  ["a key on a row it does not fit warns and sends nothing"] = function()
    local _, fake = status.opened()
    local notes = buffer.capture_notes()
    status.on(STAGED)
    buffer.feed("s")
    status.nothing_sent(fake, "add")
    assert(#notes == 1 and notes[1].level == vim.log.levels.WARN, vim.inspect(notes))
    assert(notes[1].text:find("s stages an unstaged change", 1, true), notes[1].text)
  end,

  ["<CR> on a change opens that task in the window's folder"] = function()
    local opened
    local cwd = status.folder()
    status.opened(function(oid, folder)
      opened = { oid, folder }
    end)
    status.on(UNSTAGED)
    buffer.feed("<CR>")
    assert(vim.deep_equal(opened, { MOM, cwd }), vim.inspect(opened))
  end,

  ["<CR> on a row that names no task warns"] = function()
    status.opened()
    local notes = buffer.capture_notes()
    status.on("  fake  2 commits")
    buffer.feed("<CR>")
    assert(#notes == 1 and notes[1].level == vim.log.levels.WARN, vim.inspect(notes))
  end,
}
