local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local real_dam = require("helpers.real_dam")
local wait_for = buffer.wait_for

local function answered(app, call)
  local got = helper.call(app, call, call.write and "write" or "run")
  wait_for(function()
    return got.done
  end, 20000)
  assert(not got.err, vim.inspect(got.err))
  return got.answer
end

local function captured_in_a_project(sandbox)
  local app = real_dam.app()
  require("damnit.command").define(app, { require("damnit.capture").new(app) })
  local project = vim.fs.joinpath(sandbox, "garden")
  vim.fn.mkdir(vim.fs.joinpath(project, "src"), "p")
  answered(app, { args = { "init" }, cwd = project, write = true })
  local file = vim.fs.joinpath(project, "src", "beds.lua")
  vim.fn.writefile({ "local M = {}", "-- TODO: water the seedlings", "return M" }, file)
  vim.cmd.edit(file)
  vim.bo.commentstring = "-- %s"
  vim.cmd.lcd(sandbox)
  local notes = buffer.capture_notes()
  vim.cmd("2Dam capture!")
  wait_for(function()
    return #notes > 0
  end, 20000)
  assert(notes[1].text == "captured water the seedlings", vim.inspect(notes))
  local listed = answered(app, { args = { "list" }, cwd = project }).objects
  local task = listed[1]
  assert(#listed == 1 and task.subject == "water the seedlings" and task.path == "garden/", vim.inspect(listed))
  assert(task.body == "beds.lua:2", vim.inspect(task.body))
end

return {
  ["capture from a file in a project files the task in that project with the place in its body"] = function()
    return real_dam.with_real_dam(captured_in_a_project)
  end,
}
