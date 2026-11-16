local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local list = require("helpers.list")
local prompt = require("helpers.prompt")

local MOM, MILK = list.CALL_MOM, list.OAT_MILK
local EXIT_USAGE = 2

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

local function title()
  return vim.trim(vim.api.nvim_win_get_config(assert(prompt.window())).title[1][1])
end

return {
  ["cd opens dam edit prefilled with --due and reads dam's preview after a pause"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    prompt.type("cd2026-11-20")
    same(title(), "dam edit")
    same(prompt.line(), "--due 2026-11-20")
    prompt.footer_reads("due  none → 2026-11-20")
    same(list.sent(fake, "edit", true)[1].argv, { "edit", MOM, "--due", "2026-11-20", "--preview", "-o", "json" })
  end,

  ["<CR> sends the same flags without --preview, and . sends them again on the next row"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    prompt.type("cd2026-11-20<CR>")
    same(list.wait_sent(fake, "edit", 1)[1].argv, { "edit", MOM, "--due", "2026-11-20", "-o", "json" })
    list.on("[ ] p2 buy oat milk")
    buffer.feed(".")
    same(list.wait_sent(fake, "edit", 2)[2].argv, { "edit", MILK, "--due", "2026-11-20", "-o", "json" })
  end,

  ["on three rows the float lists each row's before and after from dam's preview"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    prompt.type("3cd2026-11-20")
    same(title(), "dam edit · 3 rows")
    prompt.footer_reads("3 of 3 would change")
    same(prompt.below(), {
      "buy oat milk  due  2026-09-25 → 2026-11-20",
      "call mom      due  none → 2026-11-20",
      "pack the bag  due  none → 2026-11-20",
    })
    local preview = list.sent(fake, "edit", true)[1]
    same(preview.argv, { "edit", "-", "--due", "2026-11-20", "--expect", "3", "--preview", "-o", "json" })
  end,

  ["a date dam cannot read shows dam's message and <CR> sends nothing"] = function()
    local _, fake = list.opened()
    fake.reply("edit", fixtures.refusal("error-edit-due.json", EXIT_USAGE))
    list.on("[ ] call mom")
    prompt.type([[cd"the twelfth of never"]])
    prompt.below_reads(fixtures.decoded("error-edit-due.json").error.message)
    prompt.more("<CR>")
    vim.wait(50)
    assert(#list.sent(fake, "edit", false) == 0 and prompt.window(), vim.inspect(fake.of("edit")))
  end,

  ["cs is prefilled with the row's subject and takes one row"] = function()
    list.opened()
    local notes = buffer.capture_notes()
    list.on("[ ] call mom")
    prompt.type("cs")
    same(prompt.line(), '--subject "call mom"')
    prompt.type("<Esc>")
    buffer.feed("2cs")
    assert(prompt.window() == nil and notes[#notes].text == "cs changes one row at a time", vim.inspect(notes))
  end,

  ["cm and cP move with dam mv; gx and cc open an empty prompt for the command's flags"] = function()
    list.opened()
    list.on("[ ] call mom")
    for keys, want in pairs({ cm = "--path ", cP = "--parent ", gx = "", cc = "" }) do
      prompt.type(keys)
      same(prompt.line(), want)
      same(title(), keys == "gx" and "dam done" or (keys:sub(2, 2) == "c" and "dam edit" or "dam mv"))
      prompt.type("<Esc>")
    end
  end,

  ["--parent completes the rows on screen, their subject as menu text"] = function()
    list.opened()
    list.on("[ ] call mom")
    prompt.type("cP1")
    local buf = vim.api.nvim_win_get_buf(assert(prompt.window()))
    local source = vim.b[buf].damnit_complete
    same(source(1, ""), #"--parent ")
    local items = source(0, "")
    same(items[2], { word = "1111111", menu = "buy oat milk" })
  end,

  ["<CR> on a refused line keeps one entry in the log and announces nothing"] = function()
    local app, fake = list.opened()
    fake.reply("edit", fixtures.refusal("error-edit-due.json", EXIT_USAGE))
    local notes = buffer.capture_notes()
    list.on("[ ] call mom")
    prompt.type([[cd"the twelfth of never"]])
    prompt.below_reads(fixtures.decoded("error-edit-due.json").error.message)
    assert(#app.say:log() == 0, vim.inspect(app.say:log()))
    prompt.more("<CR>")
    vim.wait(50)
    assert(#notes == 0 and #app.say:log() == 1, vim.inspect({ notes, app.say:log() }))
  end,
}
