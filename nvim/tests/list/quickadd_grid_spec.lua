local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")

local EXIT_USAGE = 2

local function row(number)
  vim.cmd.redraw()
  local chars = {}
  for col = 1, vim.o.columns do
    table.insert(chars, vim.fn.screenstring(number, col))
  end
  return table.concat(chars)
end

local function shows(number, pattern)
  local drawn = vim.wait(1000, function()
    return row(number):find(pattern) ~= nil
  end, 10)
  assert(drawn, ("row %d never matched %q: %s"):format(number, pattern, row(number)))
end

local function opened(preview)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  for _, verb in ipairs({ "filter", "category", "remote", "label", "path" }) do
    fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", EXIT_USAGE))
  end
  fake.reply("new", preview)
  prompt.type(":Dam new<CR>buy oat milk --due 2026-11-20")
  local top = vim.api.nvim_win_get_position(assert(prompt.window()))[1]
  return top + 1, top + 2, top + 3
end

return {
  ["the float's top border holds the title and the typed line sits under it"] = function()
    local top, line = opened(fixtures.answer("new-preview.json"))
    shows(top, "╭ dam new ─")
    shows(line, "│buy oat milk %-%-due 2026%-11%-20 ")
  end,

  ["after a pause the bottom border holds dam's reading at its right end"] = function()
    local _, _, bottom = opened(fixtures.answer("new-preview.json"))
    prompt.footer_reads('task "buy oat milk" · due 2026-11-20 · errand · home/')
    shows(bottom, '─ task "buy oat milk" · due 2026%-11%-20 · errand · home/ ╯ *$')
  end,

  ["a refusal shows dam's whole words under the line and their start in the bottom border"] = function()
    local top, line = opened(fixtures.refusal("error-new-preview-due.json", EXIT_USAGE))
    prompt.footer_reads(fixtures.decoded("error-new-preview-due.json").error.message)
    shows(line + 1, '│%-%-due cannot be "the twelfth of never"; a date to set is one day or instant:')
    shows(line + 2, "│2026%-11%-20, 2026%-11%-20T09:00, today, today%+3, now%+2h, or a word such as")
    local bottom = top + vim.api.nvim_win_get_height(assert(prompt.window())) + 1
    shows(bottom, '╰.- %-%-due cannot be "the twelfth of never"; .*… ╯')
  end,
}
