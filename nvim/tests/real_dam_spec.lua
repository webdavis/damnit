local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local ui = require("helpers.ui")
local wait_for = buffer.wait_for

local tests_dir = vim.fs.dirname(debug.getinfo(1, "S").source:sub(2))
local dam_bin =
  vim.fs.normalize(vim.fs.abspath(vim.env.DAM_BIN or vim.fs.joinpath(tests_dir, "../../target/debug/dam")))

local SANDBOXED = { "DAM_CONFIG", "DAM_STORE", "HOME" }

local function in_sandbox(run)
  local sandbox = vim.fn.tempname()
  vim.fn.mkdir(sandbox, "p")
  local saved = {}
  for _, name in ipairs(SANDBOXED) do
    saved[name] = vim.env[name]
  end
  vim.env.DAM_CONFIG = vim.fs.joinpath(sandbox, "config", "dam", "config.lua")
  vim.env.DAM_STORE = vim.fs.joinpath(sandbox, "dam.db")
  vim.env.HOME = sandbox
  local ok, err = pcall(run, sandbox)
  for _, name in ipairs(SANDBOXED) do
    vim.env[name] = saved[name]
  end
  assert(ok, err)
end

local function real_app()
  local options = require("damnit.kernel.options").resolve({ dam = dam_bin })
  return require("damnit.kernel.app").new(options, require("damnit.kernel.process"))
end

local function made(app, args, cwd)
  local got = helper.call(app, { args = args, cwd = cwd }, "write")
  wait_for(function()
    return got.done
  end, 20000)
  assert(not got.err, vim.inspect(got.err))
  return got.answer.oid
end

local function task_buffer(sandbox, due)
  require("damnit").setup({ dam = dam_bin })
  local oid = made(real_app(), { "new", "plant the tomatoes", "--due", due }, sandbox)
  vim.cmd.lcd(sandbox)
  vim.cmd("Dam task " .. oid)
  wait_for(function()
    return buffer.lines()[2] == "due: " .. due
  end, 20000)
  return oid
end

local function shown_due(oid, sandbox)
  local got = helper.call(real_app(), { args = { "show", oid }, cwd = sandbox })
  wait_for(function()
    return got.done
  end, 20000)
  assert(not got.err, vim.inspect(got.err))
  return got.answer.task.due
end

local function staged_and_committed_in_status(sandbox)
  require("damnit").setup({ dam = dam_bin })
  made(real_app(), { "new", "plant the tomatoes" }, sandbox)
  vim.cmd.lcd(sandbox)
  vim.cmd("Dam status")
  local row = "  create plant the tomatoes  [subject]"
  wait_for(function()
    return vim.deep_equal(buffer.lines(), { "Unstaged (1)", row })
  end, 20000)
  vim.api.nvim_win_set_cursor(0, { 2, 0 })
  buffer.feed("s")
  wait_for(function()
    return vim.deep_equal(buffer.lines(), { "Staged (1)", row })
  end, 20000)
  ui.answering({ input = "plant" }, function()
    buffer.feed("c")
  end)
  wait_for(function()
    return vim.deep_equal(buffer.lines(), { "nothing changed, staged, waiting or unpushed" })
  end, 20000)
end

return {
  ["writing a changed due line in the task buffer changes what dam show prints"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(function(sandbox)
      local oid = task_buffer(sandbox, "2026-09-25")
      vim.api.nvim_buf_set_lines(0, 1, 2, false, { "due: 2026-10-01" })
      vim.cmd.write()
      wait_for(function()
        return not vim.bo.modified
      end, 20000)
      local due = shown_due(oid, sandbox)
      assert(due == "2026-10-01", vim.inspect(due))
    end)
  end,

  ["a refused write in the task buffer keeps it modified and shows dam's message"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(function(sandbox)
      local oid = task_buffer(sandbox, "2026-09-25")
      local notes = buffer.capture_notes()
      vim.api.nvim_buf_set_lines(0, 1, 2, false, { "due: the twelfth of never" })
      vim.cmd.write()
      wait_for(function()
        return #notes > 0
      end, 20000)
      assert(notes[1].level == vim.log.levels.ERROR and notes[1].text:find("twelfth", 1, true), vim.inspect(notes))
      assert(vim.bo.modified, "no longer modified")
      assert(shown_due(oid, sandbox) == "2026-09-25")
    end)
  end,

  ["status shows a change made with dam new; s stages it and c commits it, and the section empties"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(staged_and_committed_in_status)
  end,

  ["a task made with dam new appears in :Dam list, its child indented under it"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(function(sandbox)
      local app = real_app()
      require("damnit.command").define(app, { require("damnit.list").new(app) })
      local parent = made(app, { "new", "plant the tomatoes", "--due", "2026-09-25" }, sandbox)
      made(app, { "new", "pay rent", "--due", "2026-09-30" }, sandbox)
      made(app, { "new", "buy stakes", "--parent", parent }, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam list")
      wait_for(function()
        return #buffer.lines() == 3
      end, 20000)
      local lines = buffer.lines()
      local expected = { "[ ] plant the tomatoes 2026-09-25", "  [ ] buy stakes", "[ ] pay rent 2026-09-30" }
      assert(vim.deep_equal(lines, expected), vim.inspect(lines))
    end)
  end,

  ["the client runs the real dam list and reads an empty list"] = function()
    if vim.fn.executable(dam_bin) ~= 1 then
      return ("no dam at %s; build it with cargo build -p damnit or set DAM_BIN"):format(dam_bin)
    end
    in_sandbox(function(sandbox)
      local got = helper.call(real_app(), { args = { "list" }, cwd = sandbox })
      wait_for(function()
        return got.done
      end, 20000)
      assert(not got.err, vim.inspect(got.err))
      assert(vim.deep_equal(got.answer.objects, {}), vim.inspect(got.answer))
    end)
  end,
}
