local brief = require("damnit.handoff.brief")
local buffer = require("helpers.buffer")
local clipboard = require("helpers.clipboard")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local status = require("helpers.status")
local ui = require("helpers.ui")

local OAT, MOM = ("1"):rep(40), ("2"):rep(40)
local SHOWN = fixtures.decoded("show.json")
local NOTE_PROMPT = "note for the agent (empty for none): "

local function fake_rex(available, err)
  local rex = { name = "rex", briefs = {} }
  function rex.available()
    return available, not available and "REX_BLOCK is not set" or nil
  end
  function rex.send(text, done)
    table.insert(rex.briefs, text)
    done(err, "handed to rex: block:7")
  end
  return rex
end

local function hosts(rex)
  clipboard.fake()
  return { rex = rex or fake_rex(false), clipboard = require("damnit.handoff.clipboard").new() }
end

local function listed(options, rex)
  local app, fake = helper.new(options)
  local handoff = require("damnit.handoff").new(app, hosts(rex))
  require("damnit.command").define(app, { require("damnit.list").new(app, function() end), handoff })
  fake.reply("list", fixtures.answer("list.json"))
  fake.reply("show", fixtures.answer("show.json"))
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  vim.cmd("Dam list")
  buffer.wait_for(function()
    return #buffer.lines() > 1
  end)
  vim.fn.setreg('"', "")
  vim.api.nvim_win_set_cursor(0, { vim.fn.match(buffer.lines(), "buy oat milk") + 1, 0 })
  return fake, vim.fn.getcwd()
end

local function said(notes, text, level)
  buffer.wait_for(function()
    return #notes > 0
  end)
  assert(notes[#notes].text == text and notes[#notes].level == level, vim.inspect(notes))
end

local function shown_argv(fake)
  buffer.wait_for(function()
    return #fake.of("show") > 0
  end)
  return fake.of("show")[1]
end

return {
  [":Dam handoff <note> in a list puts the row's brief in the clipboard registers and says so"] = function()
    local fake, cwd = listed()
    local notes = buffer.capture_notes()
    vim.cmd("Dam handoff start  with the beds")
    local call = shown_argv(fake)
    assert(vim.deep_equal(call.argv, { "dam", "show", OAT, "-o", "json" }), vim.inspect(call.argv))
    assert(call.cwd == cwd, call.cwd)
    said(notes, "brief copied to the clipboard", vim.log.levels.INFO)
    assert(vim.fn.getreg('"') == brief.text(SHOWN, "start  with the beds"), vim.fn.getreg('"'))
    assert(vim.fn.getreg("+") == vim.fn.getreg('"'), vim.fn.getreg("+"))
  end,

  ["H on a list row asks for a note and hands the task to rex when rex is available"] = function()
    local rex = fake_rex(true)
    listed({}, rex)
    local notes = buffer.capture_notes()
    ui.answering({ input = "" }, function(prompts)
      buffer.feed("H")
      assert(prompts[1] == NOTE_PROMPT, vim.inspect(prompts))
    end)
    said(notes, "handed to rex: block:7", vim.log.levels.INFO)
    assert(vim.deep_equal(rex.briefs, { brief.text(SHOWN, nil) }), vim.inspect(rex.briefs))
    assert(vim.fn.getreg('"') == "", vim.fn.getreg('"'))
  end,

  ["a cancelled note hands nothing off"] = function()
    local fake = listed()
    ui.answering({}, function()
      buffer.feed("H")
    end)
    vim.wait(30)
    assert(#fake.of("show") == 0, vim.inspect(fake.of("show")))
  end,

  ["a failed rex send falls back to the clipboard and says why"] = function()
    listed({}, fake_rex(true, "no pane chosen"))
    local notes = buffer.capture_notes()
    vim.cmd("Dam handoff look")
    said(notes, "rex: no pane chosen; brief copied to the clipboard", vim.log.levels.WARN)
    assert(vim.fn.getreg('"') == brief.text(SHOWN, "look"), vim.fn.getreg('"'))
  end,

  ["host rex without rex falls back to the clipboard and says why"] = function()
    listed({ handoff = { host = "rex" } })
    local notes = buffer.capture_notes()
    vim.cmd("Dam handoff look")
    said(notes, "rex: REX_BLOCK is not set; brief copied to the clipboard", vim.log.levels.WARN)
  end,

  ["host clipboard never asks rex"] = function()
    local rex = fake_rex(true)
    listed({ handoff = { host = "clipboard" } }, rex)
    local notes = buffer.capture_notes()
    vim.cmd("Dam handoff look")
    said(notes, "brief copied to the clipboard", vim.log.levels.INFO)
    assert(#rex.briefs == 0, vim.inspect(rex.briefs))
  end,

  [":Dam handoff with no task under the cursor warns and reads nothing"] = function()
    local fake = listed()
    vim.api.nvim_win_set_cursor(0, { 1, 0 })
    vim.cmd.enew()
    local notes = buffer.capture_notes()
    vim.cmd("Dam handoff look")
    said(notes, "put the cursor on a task in a dam list, the status window or a task buffer", vim.log.levels.WARN)
    assert(#fake.of("show") == 0, vim.inspect(fake.of("show")))
  end,

  ["H on a status change hands that change's task"] = function()
    local app, fake = status.defined()
    require("damnit.handoff").new(app, hosts())
    status.open(fake)
    fake.reply("show", fixtures.answer("show.json"))
    status.on("  update  call mom back")
    ui.answering({ input = "" }, function()
      buffer.feed("H")
    end)
    assert(shown_argv(fake).argv[3] == MOM, vim.inspect(shown_argv(fake).argv))
  end,

  [":Dam handoff in a task buffer hands that task"] = function()
    local app, fake = helper.new()
    local handoff = require("damnit.handoff").new(app, hosts())
    require("damnit.command").define(app, { require("damnit.task").new(app), handoff })
    fake.reply("show", fixtures.answer("show.json"))
    vim.cmd("Dam task abcdef1")
    buffer.wait_for(function()
      return buffer.lines()[1]:find("^subject:") ~= nil
    end)
    vim.cmd("Dam handoff look")
    buffer.wait_for(function()
      return #fake.of("show") == 2
    end)
    assert(fake.of("show")[2].argv[3] == "abcdef1", vim.inspect(fake.of("show")[2].argv))
  end,

  ["H on a header row warns and reads nothing"] = function()
    local app, fake = status.defined()
    require("damnit.handoff").new(app, hosts())
    status.open(fake)
    status.on("Unstaged (1)")
    local notes = buffer.capture_notes()
    buffer.feed("H")
    said(notes, "H acts on task rows; this line holds no task", vim.log.levels.WARN)
    assert(#fake.of("show") == 0, vim.inspect(fake.of("show")))
  end,

  ["g? lists H under task rows in a list and under status in the status window"] = function()
    listed()
    buffer.feed("g?")
    local list_help = buffer.lines()
    vim.cmd.close()
    local app, fake = status.defined()
    require("damnit.handoff").new(app, hosts())
    status.open(fake)
    buffer.feed("g?")
    for group, lines in pairs({ ["task rows"] = list_help, status = buffer.lines() }) do
      local heading, h = vim.fn.index(lines, group), vim.fn.index(lines, "  H  hand the task to an agent")
      local next_heading = vim.fn.match(lines, "^\\S", heading + 1)
      assert(heading >= 0 and h > heading and (next_heading < 0 or h < next_heading), vim.inspect(lines))
    end
  end,

  ["a refused dam show is shown once and kept for :Dam messages"] = function()
    local app, fake = helper.new()
    fake.reply("show", fixtures.refusal("error-refused.json", 4))
    local notes = buffer.capture_notes()
    require("damnit.handoff").new(app, hosts()).hand_off("0000000", vim.fn.getcwd(), nil)
    buffer.wait_for(function()
      return #notes > 0
    end)
    vim.wait(30)
    assert(#notes == 1 and notes[1].level == vim.log.levels.ERROR, vim.inspect(notes))
    assert(#app.say:log() == 1, vim.inspect(app.say:log()))
  end,
}
