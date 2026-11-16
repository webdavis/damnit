local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local MILK = ("1"):rep(40)
local MOM = ("2"):rep(40)

local function task_shown(oid)
  local app, fake = helper.new()
  fake.reply("show", function(argv)
    return fixtures.answer(argv[3] == "abcdef1" and "show-related.json" or "show.json")
  end)
  fake.reply("list", fixtures.answer("list-children.json"))
  require("damnit.command").define(app, { require("damnit.task").new(app) })
  vim.cmd("Dam task " .. (oid or "abcdef1"))
  buffer.wait_for(function()
    return buffer.lines()[1]:find("^subject:") ~= nil
  end)
  return app, fake
end

local function entries()
  return vim.tbl_map(function(item)
    return { vim.api.nvim_buf_get_name(item.bufnr), item.text }
  end, vim.fn.getloclist(0))
end

local function app_with_peek()
  local app, fake = helper.new()
  fake.reply("show", fixtures.answer("show.json"))
  return app, fake, require("damnit.peek").new(app, function() end)
end

return {
  ["gO lists the parent, the children and what the task waits on in the location list"] = function()
    local _, fake = task_shown()
    local task_win = vim.api.nvim_get_current_win()
    buffer.feed("gO")
    buffer.wait_for(function()
      return #vim.fn.getloclist(task_win) > 0
    end)
    local list_call = fake.of("list")[1]
    assert(
      vim.deep_equal(list_call.argv, { "dam", "list", "--request", "-", "-o", "json" }),
      vim.inspect(list_call.argv)
    )
    assert(vim.deep_equal(vim.json.decode(list_call.stdin), { parent = "<id-1>" }), list_call.stdin)
    vim.api.nvim_set_current_win(task_win)
    local expected = {
      { "dam://task/" .. MOM, "parent: plant the tomatoes" },
      { "dam://task/<id-1>", "child: buy the seeds" },
      { "dam://task/<id-3>", "child: dig the beds" },
      { "dam://task/" .. MILK, "waits on: plant the tomatoes" },
    }
    assert(vim.deep_equal(entries(), expected), vim.inspect(entries()))
    assert(vim.fn.getloclist(task_win, { winid = 0 }).winid ~= 0, "the location list window is not open")
  end,

  ["gO on a task with nothing related says so and opens no list"] = function()
    local notes = buffer.capture_notes()
    local _, fake = task_shown("0123456")
    fake.reply("list", { code = 0, stdout = '{"objects":[]}', stderr = "" })
    vim.fn.setloclist(0, {}, "f")
    buffer.feed("gO")
    buffer.wait_for(function()
      return #notes > 0
    end)
    assert(notes[1].text:find("no parent, children or dependencies", 1, true), vim.inspect(notes))
    assert(#vim.fn.getloclist(0) == 0)
  end,

  ["gd opens the task whose oid is under the cursor"] = function()
    task_shown()
    vim.bo.modified = false
    vim.api.nvim_buf_set_lines(0, -1, -1, false, { "see 1234abc for the soil" })
    vim.bo.modified = false
    vim.api.nvim_win_set_cursor(0, { vim.api.nvim_buf_line_count(0), 6 })
    buffer.feed("gd")
    assert(vim.api.nvim_buf_get_name(0) == "dam://task/1234abc", vim.api.nvim_buf_get_name(0))
  end,

  ["gd on a word that is no oid opens nothing"] = function()
    local notes = buffer.capture_notes()
    task_shown()
    buffer.feed("gd")
    assert(vim.api.nvim_buf_get_name(0) == "dam://task/abcdef1")
    assert(notes[1] and notes[1].text:find("no oid under the cursor", 1, true), vim.inspect(notes))
  end,

  ["K on an oid in a task buffer peeks at that task"] = function()
    local app, fake, peek = app_with_peek()
    require("damnit.command").define(app, { require("damnit.task").new(app, peek) })
    vim.cmd("Dam task abcdef1")
    buffer.wait_for(function()
      return buffer.lines()[1]:find("^subject:") ~= nil
    end)
    vim.api.nvim_buf_set_lines(0, -1, -1, false, { "see 1234abc for the soil" })
    vim.api.nvim_win_set_cursor(0, { vim.api.nvim_buf_line_count(0), 6 })
    buffer.feed("K")
    buffer.wait_for(function()
      return #fake.of("show") > 1
    end)
    assert(fake.of("show")[2].argv[3] == "1234abc", fake.of("show")[2].argv[3])
  end,

  ["K on a word that is no oid in a task buffer peeks at nothing"] = function()
    local notes = buffer.capture_notes()
    local app, fake, peek = app_with_peek()
    require("damnit.command").define(app, { require("damnit.task").new(app, peek) })
    vim.cmd("Dam task abcdef1")
    buffer.wait_for(function()
      return buffer.lines()[1]:find("^subject:") ~= nil
    end)
    buffer.feed("K")
    assert(#fake.of("show") == 1, vim.inspect(fake.of("show")))
    assert(
      vim.deep_equal(
        vim.tbl_map(function(note)
          return note.text
        end, notes),
        { "no oid under the cursor to peek at" }
      ),
      vim.inspect(notes)
    )
  end,

  ["g? in a task buffer lists gO, gd and K under the task buffer heading"] = function()
    local app, _, peek = app_with_peek()
    require("damnit.command").define(app, { require("damnit.task").new(app, peek) })
    vim.cmd("Dam task abcdef1")
    buffer.wait_for(function()
      return buffer.lines()[1]:find("^subject:") ~= nil
    end)
    buffer.feed("g?")
    local expected = {
      "task buffer",
      "  gO  list the parent, children and dependencies",
      "  gd  open the task whose oid is under the cursor",
      "  K  peek at the task, or enter the peek",
      "every window",
      "  g?  list these keys",
    }
    assert(vim.deep_equal(buffer.lines(), expected), vim.inspect(buffer.lines()))
  end,
}
