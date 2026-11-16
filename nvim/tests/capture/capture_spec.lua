local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local prompt = require("helpers.prompt")

local EXIT_USAGE = 2
local PLUGIN = vim.fs.joinpath(
  vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)))),
  "plugin",
  "damnit.lua"
)
local SOURCE = { "local M = {}", "", "  -- TODO(sam): free the buffer", "  --   before returning", "return M" }

local function previewing(argv)
  return vim.tbl_contains(argv, "--preview")
end

local function defined(made)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.capture").new(app) })
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

local function source_file()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(vim.fs.joinpath(dir, "src"), "p")
  local path = vim.fs.joinpath(dir, "src", "beds.lua")
  vim.fn.writefile(SOURCE, path)
  vim.cmd.edit(path)
  vim.bo.commentstring, vim.bo.comments = "-- %s", "s:--[[,m: ,e:]],:--"
  vim.wo.number = false
  vim.cmd.lcd(dir)
  return vim.fs.dirname(vim.api.nvim_buf_get_name(0))
end

local function written(fake)
  buffer.wait_for(function()
    return #vim.tbl_filter(function(call)
      return not previewing(call.argv)
    end, fake.of("new")) > 0
  end)
  local call = fake.of("new")[#fake.of("new")]
  return call.argv, call.cwd
end

return {
  ["dam runs in the file's folder"] = function()
    local _, fake = defined()
    local folder = source_file()
    vim.cmd("3Dam capture!")
    local _, cwd = written(fake)
    assert(cwd == folder, cwd .. " is not " .. folder)
  end,

  ["capture opens the prompt with the subject before the cursor and the place after it"] = function()
    local _, fake = defined()
    source_file()
    prompt.type(":3,4Dam capture<CR> now")
    assert(prompt.line() == 'free the buffer before returning now --body "beds.lua:3"', prompt.line())
    local config = vim.api.nvim_win_get_config(assert(prompt.window()))
    assert(vim.trim(config.title[1][1]) == "dam new", vim.inspect(config.title))
    prompt.footer_reads('task "buy oat milk" · due 2026-11-20 · errand · home/')
    local want = { "dam", "new", "free the buffer before returning now", "--body", "beds.lua:3", "--preview" }
    assert(vim.deep_equal(vim.list_slice(fake.of("new")[1].argv, 1, 6), want), vim.inspect(fake.of("new")))
  end,

  ["capture shows dam's reading of the prefilled line before anything is typed"] = function()
    local _, fake = defined()
    source_file()
    vim.cmd("3Dam capture")
    prompt.footer_reads('task "buy oat milk" · due 2026-11-20 · errand · home/')
    local want = { "dam", "new", "free the buffer", "--body", "beds.lua:3", "--preview", "-o", "json" }
    assert(vim.deep_equal(fake.of("new")[1].argv, want), vim.inspect(fake.of("new")))
  end,

  ["the prompt opens under the captured line at the source window's text width"] = function()
    defined()
    source_file()
    vim.wo.number = true
    local source = vim.api.nvim_get_current_win()
    vim.cmd("3Dam capture")
    local win = assert(prompt.window())
    local config = vim.api.nvim_win_get_config(win)
    assert(config.win == source and config.anchor == "NW" and config.row == 3, vim.inspect(config))
    local textoff = vim.fn.getwininfo(source)[1].textoff
    assert(config.col == textoff and config.width == vim.o.columns - textoff - 2, vim.inspect(config))
    assert(vim.api.nvim_win_get_cursor(source)[1] == 3)
  end,

  ["the prompt opens above the captured line when the window has no room below"] = function()
    defined()
    source_file()
    local source = vim.api.nvim_get_current_win()
    vim.api.nvim_buf_set_lines(0, 0, -1, false, vim.split(string.rep("-- TODO: weed the bed\n", 60), "\n"))
    vim.cmd("normal! L")
    local line = vim.fn.winline()
    vim.cmd("Dam capture")
    local config = vim.api.nvim_win_get_config(assert(prompt.window()))
    assert(config.win == source and config.anchor == "SW" and config.row == line - 1, vim.inspect(config))
  end,

  ["the row above the prompt's top border is the captured comment line"] = function()
    defined()
    source_file()
    vim.cmd("3Dam capture")
    local top = vim.api.nvim_win_get_position(assert(prompt.window()))[1] + 1
    local grid = require("helpers.grid")
    buffer.wait_for(function()
      return grid.row(top):find("^╭ dam new") ~= nil
    end)
    assert(grid.row(top - 1):find("^  %-%- TODO%(sam%): free the buffer"), grid.row(top - 1))
  end,

  ["<CR> in the prompt sends the line and says what was captured"] = function()
    local _, fake = defined()
    source_file()
    local notes = buffer.capture_notes()
    prompt.type(":3Dam capture<CR><CR>")
    local argv = written(fake)
    local want = { "dam", "new", "free the buffer", "--body", "beds.lua:3", "-o", "json" }
    assert(vim.deep_equal(argv, want), vim.inspect(argv))
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "captured buy oat milk", vim.inspect(notes))
  end,

  ["capture with a bang sends dam new at once"] = function()
    local _, fake = defined()
    source_file()
    vim.api.nvim_win_set_cursor(0, { 3, 0 })
    vim.cmd("Dam capture!")
    local argv = written(fake)
    local want = { "dam", "new", "free the buffer", "--body", "beds.lua:3", "-o", "json" }
    assert(vim.deep_equal(argv, want), vim.inspect(argv))
    assert(prompt.window() == nil and #vim.tbl_filter(
      previewing,
      vim.tbl_map(function(call)
        return call.argv
      end, fake.of("new"))
    ) == 0)
  end,

  ["an empty line warns and sends nothing"] = function()
    local _, fake = defined()
    source_file()
    local notes = buffer.capture_notes()
    vim.cmd("2Dam capture")
    vim.cmd("2Dam capture!")
    vim.wait(30)
    assert(#fake.of("new") == 0 and prompt.window() == nil, vim.inspect(fake.of("new")))
    assert(
      #notes == 2 and notes[1].level == vim.log.levels.WARN and notes[2].level == vim.log.levels.WARN,
      vim.inspect(notes)
    )
    assert(notes[1].text == "nothing to capture on line 2", notes[1].text)
  end,

  ["a selection mapped to <Plug>(dam-capture) captures the selected lines"] = function()
    defined()
    source_file()
    vim.cmd.source(PLUGIN)
    vim.keymap.set("x", "gC", "<Plug>(dam-capture)", { remap = true })
    prompt.type("3GVjgC")
    assert(prompt.line() == 'free the buffer before returning --body "beds.lua:3"', prompt.line())
  end,

  ["under a token the echo says it waits for the owner"] = function()
    local _, fake = defined(fixtures.decoded("sent.json"))
    source_file()
    local notes = buffer.capture_notes()
    vim.cmd("3Dam capture!")
    written(fake)
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text == "sent for the owner: <id-1>", vim.inspect(notes))
  end,
}
