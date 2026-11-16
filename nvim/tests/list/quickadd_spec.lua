local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")

local EXIT_USAGE = 2
local CALL_MOM = "2222222222222222222222222222222222222222"

local function previewing(argv)
  return vim.tbl_contains(argv, "--preview")
end

local function defined(options, made)
  local app, fake = helper.new(options)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", fixtures.answer("list.json"))
  for _, verb in ipairs({ "filter", "category", "remote", "label", "path" }) do
    fake.reply(verb, fixtures.refusal("error-unknown-subcommand.json", EXIT_USAGE))
  end
  fake.reply("new", function(argv)
    if previewing(argv) then
      return fixtures.answer("new-preview.json")
    end
    return { code = 0, stdout = vim.json.encode(made or fixtures.decoded("new.json")), stderr = "" }
  end)
  return app, fake
end

local function sent(fake, preview)
  return vim.tbl_filter(function(call)
    return previewing(call.argv) == preview
  end, fake.of("new"))
end

local function listed(fake, view)
  vim.cmd("Dam list" .. (view and (" " .. view) or ""))
  buffer.wait_for(function()
    return #fake.of("list") > 0 and #buffer.lines() > 1
  end)
end

return {
  [":Dam new reads each pause with dam new --preview in this folder and shows dam's reading"] = function()
    local _, fake = defined()
    local cwd = require("helpers.status").folder()
    prompt.type(":Dam new<CR>buy oat milk --due 2026-11-20")
    local config = vim.api.nvim_win_get_config(assert(prompt.window()))
    assert(vim.trim(config.title[1][1]) == "dam new", vim.inspect(config.title))
    assert(config.relative == "editor", config.relative)
    prompt.footer_reads('task "buy oat milk" · due 2026-11-20 · errand · home/')
    local previews = sent(fake, true)
    assert(#previews == 1 and previews[1].cwd == cwd, vim.inspect(previews))
    local want = { "dam", "new", "buy oat milk", "--due", "2026-11-20", "--preview", "-o", "json" }
    assert(vim.deep_equal(previews[1].argv, want), vim.inspect(previews[1].argv))
  end,

  ["a refusal is dam's message in DiagnosticError and <CR> writes nothing"] = function()
    local _, fake = defined()
    fake.reply("new", fixtures.refusal("error-new-preview-due.json", EXIT_USAGE))
    prompt.type(":Dam new<CR>buy oat milk --due the twelfth of never")
    local message = fixtures.decoded("error-new-preview-due.json").error.message
    prompt.footer_reads(message)
    local _, group = prompt.footer()
    assert(group == "DiagnosticError", group)
    prompt.more("<CR>")
    vim.wait(50)
    assert(#sent(fake, false) == 0 and prompt.window(), vim.inspect(fake.of("new")))
  end,

  ["<CR> sends the same words without --preview and says what dam made"] = function()
    local _, fake = defined()
    local notes = buffer.capture_notes()
    prompt.type(":Dam new<CR>buy oat milk --due 2026-11-20<CR>")
    buffer.wait_for(function()
      return #notes > 0
    end)
    local writes = sent(fake, false)
    local want = { "dam", "new", "buy oat milk", "--due", "2026-11-20", "-o", "json" }
    assert(#writes == 1 and vim.deep_equal(writes[1].argv, want), vim.inspect(writes))
    assert(notes[1].text == "new: buy oat milk", vim.inspect(notes))
    assert(prompt.window() == nil)
  end,

  [":Dam new with words sends them at once"] = function()
    local _, fake = defined()
    vim.cmd([[Dam new buy "oat milk" --due fri]])
    buffer.wait_for(function()
      return #sent(fake, false) == 1
    end)
    local want = { "dam", "new", "buy oat milk", "--due", "fri", "-o", "json" }
    assert(vim.deep_equal(sent(fake, false)[1].argv, want), vim.inspect(fake.of("new")))
    assert(#sent(fake, true) == 0 and prompt.window() == nil)
  end,

  ["a in a list whose view has one path puts --path after the cursor"] = function()
    local _, fake = defined({ views = { home = { path = "home/" } } })
    listed(fake, "home")
    local list = vim.api.nvim_get_current_win()
    prompt.type("abuy")
    assert(prompt.line() == "buy --path home/", prompt.line())
    assert(vim.api.nvim_win_get_config(assert(prompt.window())).win == list, "not anchored to the list")
  end,

  ["o puts --parent with the row's short oid after the cursor"] = function()
    local _, fake = defined()
    listed(fake)
    local row = vim.fn.index(buffer.lines(), "[ ] call mom") + 1
    vim.api.nvim_win_set_cursor(0, { row, 0 })
    prompt.type("obuy")
    assert(prompt.line() == "buy --parent 2222222", prompt.line())
  end,

  ["a new task the list shows takes the cursor"] = function()
    local made = fixtures.decoded("new.json")
    made.oid, made.subject = CALL_MOM, "call mom"
    local _, fake = defined({}, made)
    listed(fake)
    vim.api.nvim_win_set_cursor(0, { #buffer.lines(), 0 })
    local notes = buffer.capture_notes()
    prompt.type("acall mom<CR>")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "new: call mom", vim.inspect(notes))
    assert(buffer.lines()[vim.api.nvim_win_get_cursor(0)[1]] == "[ ] call mom", vim.inspect(buffer.lines()))
  end,

  ["a new task the list does not show is said so"] = function()
    local made = fixtures.decoded("new.json")
    made.oid = string.rep("9", 40)
    local _, fake = defined({}, made)
    listed(fake)
    local notes = buffer.capture_notes()
    prompt.type("abuy oat milk<CR>")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "new: buy oat milk; not in this view", vim.inspect(notes))
  end,

  ["under a token the echo names the item sent for the owner"] = function()
    local _, fake = defined({}, fixtures.decoded("sent.json"))
    local notes = buffer.capture_notes()
    vim.cmd("Dam new plant the tomatoes")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(#sent(fake, false) == 1 and notes[1].text == "sent for the owner: <id-1>", vim.inspect(notes))
  end,
}
