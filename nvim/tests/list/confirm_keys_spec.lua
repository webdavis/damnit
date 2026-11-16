local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local list = require("helpers.list")

local MOM, MILK = list.CALL_MOM, list.OAT_MILK
local EXIT_REFUSED = 4

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

local function float_lines()
  return buffer.lines(vim.api.nvim_win_get_buf(assert(list.confirm_window())))
end

return {
  ["dd on one row shows dam rm's preview and sends nothing until y"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("dd")
    list.wait_confirm(fake, "rm")
    same(list.sent(fake, "rm", true)[1].argv, { "rm", MOM, "--preview", "-o", "json" })
    same(float_lines()[#float_lines()], "2 of 2 would change")
    assert(#list.sent(fake, "rm", false) == 0, "it removed before the yes")
    buffer.feed("y")
    same(list.wait_sent(fake, "rm", 1)[1].argv, { "rm", MOM, "-o", "json" })
  end,

  ["n keeps the rows and sends nothing"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("dd")
    list.wait_confirm(fake, "rm")
    buffer.feed("n")
    vim.wait(30)
    assert(#list.sent(fake, "rm", false) == 0 and not list.confirm_window(), vim.inspect(fake.of("rm")))
  end,

  ["d in visual mode removes the selected rows in one call"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("Vjd")
    list.wait_confirm(fake, "rm")
    buffer.feed("y")
    local writes = list.wait_sent(fake, "rm", 1)
    same(writes, { { argv = { "rm", "-", "--expect", "2", "-o", "json" }, stdin = MOM .. "\n" .. MILK .. "\n" } })
  end,

  ["U previews dam restore, then sets the row back"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("U")
    list.wait_confirm(fake, "restore")
    same(list.sent(fake, "restore", true)[1].argv, { "restore", MOM, "--preview", "-o", "json" })
    buffer.feed("<CR>")
    same(list.wait_sent(fake, "restore", 1)[1].argv, { "restore", MOM, "-o", "json" })
  end,

  ["a refused write lands as an error on each row dam names, once: no notification, one entry in the log"] = function()
    local app, fake = list.opened()
    local notes = buffer.capture_notes()
    fake.reply("done", fixtures.refusal("error-done-open-children.json", EXIT_REFUSED))
    list.on("[ ] p2 buy oat milk")
    buffer.feed("x")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) > 0
    end)
    local shown = vim.diagnostic.get(0)
    same(
      vim.tbl_map(function(d)
        return d.lnum
      end, shown),
      { 1, 2 }
    )
    assert(shown[1].message:find("pass --force", 1, true), shown[1].message)
    assert(#fake.of("list") == 1, "the list was read again")
    vim.wait(20)
    assert(#notes == 0, vim.inspect(notes))
    assert(#app.say:log() == 1, vim.inspect(app.say:log()))
  end,

  ["a refusal that names no row lands on the pressed row, and the next write clears it"] = function()
    local _, fake = list.opened()
    fake.reply("done", fixtures.refusal("error-expect.json", EXIT_REFUSED))
    list.on("[ ] call mom")
    buffer.feed("x")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) == 1
    end)
    same(vim.diagnostic.get(0)[1].lnum, vim.fn.line(".") - 1)
    buffer.feed("X")
    buffer.wait_for(function()
      return #vim.diagnostic.get(0) == 0
    end)
  end,
}
