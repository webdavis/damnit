local buffer = require("helpers.buffer")
local helper = require("helpers.app")
local ui = require("helpers.ui")
local wait_for = buffer.wait_for

local real_dam = require("helpers.real_dam")
local dam_bin, with_real_dam, real_app, made = real_dam.bin, real_dam.with_real_dam, real_dam.app, real_dam.made

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
  local row = "  create  plant the tomatoes"
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

local function listed_in_a_project(sandbox)
  local app = real_app()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  local project = vim.fs.joinpath(sandbox, "garden")
  vim.fn.mkdir(project, "p")
  local init = helper.call(app, { args = { "init" }, cwd = project }, "write")
  wait_for(function()
    return init.done
  end, 20000)
  assert(not init.err, vim.inspect(init.err))
  made(app, { "new", "plant the tomatoes", "--due", "2026-09-25", "--labels", "seeds" }, project)
  vim.cmd.lcd(project)
  vim.cmd("Dam list")
  wait_for(function()
    return vim.wo.winbar:find("garden/", 1, true) ~= nil and #buffer.lines() == 1
  end, 20000)
  local row = require("helpers.grid").row(2)
  assert(row:find("^! %[ %] plant the tomatoes +#seeds  %d+d late  2026%-09%-25$"), row)
  assert(require("helpers.grid").row(1):find(" default · garden/ · 1 task · 1 overdue", 1, true))
end

return {
  ["in a project :Dam list shows the project in the winbar and the due at the window's right edge"] = function()
    return with_real_dam(listed_in_a_project)
  end,

  ["writing a changed due line in the task buffer changes what dam show prints"] = function()
    return with_real_dam(function(sandbox)
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
    return with_real_dam(function(sandbox)
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
    return with_real_dam(staged_and_committed_in_status)
  end,

  ["a task made with dam new appears in :Dam list, its child indented under it"] = function()
    return with_real_dam(function(sandbox)
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
      local expected = { "[ ] plant the tomatoes", "  [ ] buy stakes", "[ ] pay rent" }
      assert(vim.deep_equal(lines, expected), vim.inspect(lines))
    end)
  end,

  ["a row shows how many days until its due, as dam counts them"] = function()
    return with_real_dam(function(sandbox)
      local app = real_app()
      require("damnit.command").define(app, { require("damnit.list").new(app) })
      made(app, { "new", "sow beans", "--due", "today+3" }, sandbox)
      made(app, { "new", "water seedlings", "--due", "today-2" }, sandbox)
      vim.cmd.lcd(sandbox)
      vim.cmd("Dam list")
      wait_for(function()
        return #buffer.lines() == 2
      end, 20000)
      local rows = require("helpers.grid").row(2) .. "\n" .. require("helpers.grid").row(3)
      assert(rows:find("sow beans +in 3d ") and rows:find("water seedlings +2d late "), rows)
    end)
  end,

  ["the client runs the real dam list and reads an empty list"] = function()
    return with_real_dam(function(sandbox)
      local got = helper.call(real_app(), { args = { "list" }, cwd = sandbox })
      wait_for(function()
        return got.done
      end, 20000)
      assert(not got.err, vim.inspect(got.err))
      assert(vim.deep_equal(got.answer.objects, {}), vim.inspect(got.answer))
    end)
  end,
}
