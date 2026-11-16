local buffer = require("helpers.buffer")
local status = require("helpers.status")

local MOM = ("2"):rep(40)
local UNSTAGED = "  update  call mom back"
local STAGED = "  create  water the plants"
local ITEM = "  new       call the plumber"
local SET = "  set of 2"

local function confirming(keys)
  buffer.feed(keys)
  buffer.wait_for(function()
    return vim.bo.filetype == "damconfirm"
  end)
end

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

  ["dd on a waiting item shows it in the confirm float and declines it only after y"] = function()
    local _, fake = status.opened()
    status.on(ITEM)
    confirming("dd")
    assert(
      vim.deep_equal(buffer.lines(), { "new  call the plumber  from tool", "", "1 waiting item would be declined" })
    )
    buffer.feed("n")
    status.nothing_sent(fake, "rm")
    confirming("dd")
    buffer.feed("y")
    assert(vim.deep_equal(status.sent(fake, "rm"), { "rm", "<id-2>" }), vim.inspect(status.sent(fake, "rm")))
  end,

  ["dd on a set declines the set after y"] = function()
    local _, fake = status.opened()
    status.on(SET)
    confirming("dd")
    assert(buffer.lines()[3] == "a set of 2 would be declined", vim.inspect(buffer.lines()))
    buffer.feed("y")
    assert(vim.deep_equal(status.sent(fake, "rm"), { "rm", "<id-3>" }), vim.inspect(status.sent(fake, "rm")))
  end,

  ["S stages everything from any row"] = function()
    local _, fake = status.opened()
    status.on(STAGED)
    buffer.feed("S")
    assert(vim.deep_equal(status.sent(fake, "add"), { "add", "-A" }), vim.inspect(status.sent(fake, "add")))
  end,

  ["U on a change shows dam restore --preview, then sets it back after y"] = function()
    local _, fake = status.opened()
    status.on(UNSTAGED)
    confirming("U")
    local preview = fake.of("restore")[1].argv
    assert(vim.deep_equal(preview, { "dam", "restore", MOM, "--preview", "-o", "json" }), vim.inspect(preview))
    buffer.feed("y")
    assert(vim.deep_equal(status.sent(fake, "restore"), { "restore", MOM }), vim.inspect(fake.of("restore")))
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

  ["a refused stage lands as an error in the window, once: no notification, one entry in the log"] = function()
    local app, fake = status.opened()
    fake.reply("add", require("helpers.fixtures").refusal("error-expect.json", 4))
    local notes = buffer.capture_notes()
    status.on(UNSTAGED)
    buffer.feed("s")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    vim.wait(20)
    assert(#vim.diagnostic.get(0) == 1 and vim.diagnostic.get(0)[1].lnum == 0, vim.inspect(vim.diagnostic.get(0)))
    assert(#notes == 0 and #app.say:log() == 1, vim.inspect({ notes, app.say:log() }))
  end,

  ["a refusal that names a change's oid lands on that change's row"] = function()
    local _, fake = status.opened()
    local refusal = require("helpers.fixtures").decoded("error-expect.json")
    refusal.error.oids = { MOM }
    fake.reply("add", { code = 4, stdout = "", stderr = vim.json.encode(refusal) })
    status.on(UNSTAGED)
    buffer.feed("s")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    assert(vim.diagnostic.get(0)[1].lnum == vim.api.nvim_win_get_cursor(0)[1] - 1, vim.inspect(vim.diagnostic.get(0)))
  end,
}
