local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local OID = "abcdef1"

local function opened(reply)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.task").new(app) })
  fake.reply("show", fixtures.answer(reply or "show.json"))
  return app, fake
end

local function folder()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  return vim.fn.getcwd()
end

local function shown(command)
  vim.cmd(command or ("Dam task " .. OID))
  buffer.wait_for(function()
    return buffer.lines()[1]:find("^subject:") ~= nil
  end)
end

local function virtual_lines()
  local texts = {}
  for _, namespace in pairs(vim.api.nvim_get_namespaces()) do
    for _, mark in ipairs(vim.api.nvim_buf_get_extmarks(0, namespace, 0, -1, { details = true })) do
      for _, chunks in ipairs(mark[4].virt_lines or {}) do
        table.insert(texts, chunks[1][1])
      end
    end
  end
  return texts
end

return {
  [":Dam task opens dam://task/<oid> with the header dam show printed"] = function()
    local _, fake = opened()
    shown()
    assert(vim.api.nvim_buf_get_name(0) == "dam://task/" .. OID, vim.api.nvim_buf_get_name(0))
    assert(vim.bo.filetype == "damtask", vim.bo.filetype)
    assert(vim.deep_equal(fake.of("show")[1].argv, { "dam", "show", OID, "-o", "json" }))
    assert(buffer.lines()[2] == "due: 2026-09-25", vim.inspect(buffer.lines()))
    assert(not vim.bo.modified, "a freshly read task is modified")
  end,

  ["the task buffer shows its short oid, path, author and overdue in the winbar, not above line 1"] = function()
    opened()
    shown()
    local winbar = require("helpers.grid").row(1)
    assert(vim.startswith(winbar, " <id-1> · home/ · by owner · overdue "), winbar)
    assert(require("helpers.grid").row(2):find("^subject: plant the tomatoes"), require("helpers.grid").row(2))
    assert(vim.deep_equal(virtual_lines(), {}), vim.inspect(virtual_lines()))
  end,

  ["a narrow task window keeps the oid and path in its winbar"] = function()
    opened()
    shown()
    vim.cmd("vertical 16split")
    local winbar = require("helpers.grid").row(1, 16)
    assert(vim.startswith(winbar, " <id-1> · home/"), winbar)
  end,

  ["the winbar stays when the task is shown again"] = function()
    opened()
    shown()
    local buf = vim.api.nvim_get_current_buf()
    vim.cmd.enew()
    vim.api.nvim_win_set_buf(0, buf)
    assert(vim.startswith(require("helpers.grid").row(1), " <id-1> · home/"), require("helpers.grid").row(1))
  end,

  ["the task buffer runs dam in the folder it was opened from"] = function()
    local _, fake = opened()
    local opened_in = folder()
    shown()
    folder()
    fake.reply("edit", fixtures.answer("edit.json"))
    vim.api.nvim_buf_set_lines(0, 1, 2, false, { "due: 2026-10-01" })
    vim.cmd.write()
    buffer.wait_for(function()
      return #fake.of("edit") == 1
    end)
    assert(fake.of("show")[1].cwd == opened_in, fake.of("show")[1].cwd)
    assert(fake.of("edit")[1].cwd == opened_in, fake.of("edit")[1].cwd)
  end,

  [":Dam task from a file runs dam in that file's folder"] = function()
    local _, fake = opened()
    folder()
    local project = vim.fn.resolve(vim.fn.tempname())
    vim.fn.mkdir(project, "p")
    vim.cmd.edit(vim.fs.joinpath(project, "notes.txt"))
    shown()
    assert(fake.of("show")[1].cwd == project, fake.of("show")[1].cwd)
  end,

  [":Dam task without an oid says how to give one"] = function()
    local _, fake = opened()
    local notes = buffer.capture_notes()
    vim.cmd("Dam task")
    assert(#fake.of("show") == 0, "dam show ran without an oid")
    assert(notes[1] and notes[1].text:find(":Dam task <oid>", 1, true), vim.inspect(notes))
  end,

  ["an event shows start, end and location"] = function()
    opened("show-event.json")
    shown()
    local lines = buffer.lines()
    assert(lines[2] == "start: 2026-09-25T09:00:00+00:00" and lines[4] == "location: room 4", vim.inspect(lines))
  end,
}
