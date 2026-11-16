local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local grid = require("helpers.grid")
local helper = require("helpers.app")

local DATE = "%d%d%d%d%-%d%d%-%d%d"

local LONG = ("a subject far too long to sit beside its columns "):rep(2)

local function with_a_long_subject()
  local answer = fixtures.decoded("list-columns.json")
  answer.objects[2].subject = LONG
  return { code = 0, stdout = vim.json.encode(answer), stderr = "" }
end

local function opened(list, config, options)
  local app, fake = helper.new(options)
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", type(list) == "table" and list or fixtures.answer(list or "list-columns.json"))
  fake.reply("config", fixtures.answer(config or "config-list-project.json"))
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, "p")
  vim.cmd.lcd(dir)
  vim.cmd("Dam list" .. (options and " everywhere" or ""))
  buffer.wait_for(function()
    return #buffer.lines() > 1 and vim.wo.winbar:find("tasks", 1, true) ~= nil
  end)
  return fake, vim.fn.getcwd()
end

local function rows_on_screen()
  local rows = {}
  for n = 1, #buffer.lines() do
    rows[n] = grid.row(n + 1)
  end
  return rows
end

return {
  ["the list reads dam config list once per folder and shows the project in the winbar"] = function()
    local fake, cwd = opened()
    buffer.feed("r")
    buffer.wait_for(function()
      return #fake.of("list") == 2
    end)
    vim.wait(30)
    assert(#fake.of("config") == 1, vim.inspect(fake.of("config")))
    assert(vim.deep_equal(fake.of("config")[1].argv, { "dam", "config", "list", "-o", "json" }))
    assert(fake.of("config")[1].cwd == cwd, fake.of("config")[1].cwd)
    assert(grid.row(1):find(" default · app/ · 6 tasks · 2 overdue · 1 blocked", 1, true), grid.row(1))
  end,

  ["outside a project the winbar names the folder dam ran in"] = function()
    local _, cwd = opened("list-columns.json", "config-list.json")
    local folder = vim.fs.basename(cwd)
    assert(grid.row(1):find(" default · " .. folder .. " · 6 tasks", 1, true), grid.row(1))
  end,

  ["each row with a value on the right has one right-aligned mark"] = function()
    opened()
    local namespace = vim.api.nvim_get_namespaces()["damnit"]
    local aligned = vim.tbl_filter(function(mark)
      return mark[4].virt_text_pos == "right_align"
    end, vim.api.nvim_buf_get_extmarks(0, namespace, 0, -1, { details = true }))
    assert(#aligned == 4, vim.inspect(aligned))
  end,

  ["on every row with a due the due ends at the window's last column"] = function()
    opened()
    local dated = 0
    for _, row in ipairs(rows_on_screen()) do
      if row:find(DATE) then
        dated = dated + 1
        assert(row:find(DATE .. "$"), row)
      end
    end
    assert(dated == 4, vim.inspect(rows_on_screen()))
  end,

  ["a long subject is never covered"] = function()
    opened()
    local subject = "fix the login redirect loop that sends people back to log in"
    local row = rows_on_screen()[3]
    assert(row:find("[ ] " .. subject .. "  ", 1, true), row)
    assert(row:find("2030%-11%-16$"), row)
  end,

  ["at 80 columns the narrower columns drop and the due stays"] = function()
    opened()
    local screen = table.concat(rows_on_screen(), "\n")
    assert(not screen:find("#urgent", 1, true) and not screen:find("effort=deep", 1, true), screen)
    assert(screen:find("2030-11-18", 1, true), screen)
  end,

  ["a narrower window drops the category and keeps the due at its last column"] = function()
    opened("list-tree.json")
    assert(grid.row(5):find("context=errand", 1, true), grid.row(5))
    vim.cmd("vertical 40split")
    vim.api.nvim_exec_autocmds("WinResized", {})
    buffer.wait_for(function()
      return not grid.row(5, 40):find("context=errand", 1, true)
    end)
    assert(grid.row(5, 40):find("2026%-09%-25$"), grid.row(5, 40))
  end,

  ["overdue and blocked rows show their sign"] = function()
    opened()
    local rows = rows_on_screen()
    assert(rows[1]:sub(1, 1) == "!" and rows[4]:sub(1, 1) == "~", vim.inspect(rows))
  end,

  ["a long subject ends in an ellipsis and its due still ends at the last column"] = function()
    opened(with_a_long_subject())
    local rows = rows_on_screen()
    assert(rows[3]:find("too long to sit beside… 45m  deadline 2030%-11%-20  2030%-11%-16$"), rows[3])
    local starts = {}
    for _, row in ipairs(rows) do
      if row:find(DATE .. "$") then
        starts[#starts + 1] = vim.fn.strdisplaywidth(row) - 10
      end
    end
    assert(#starts == 4 and #vim.fn.uniq(starts) == 1, vim.inspect(rows))
  end,

  ["rows never wrap, even in a narrow window"] = function()
    opened(with_a_long_subject())
    vim.cmd("vertical 29split")
    vim.api.nvim_exec_autocmds("WinResized", {})
    assert(vim.wo.wrap == false)
    buffer.wait_for(function()
      return grid.row(4, 29):find("…  ?2030%-11%-16$") ~= nil
    end)
    assert(grid.row(5, 29):find("^~ %[ %] p2 send invites +$"), grid.row(5, 29))
    assert(grid.row(2, 29):find("^! %[ %] p1 plan the… 2026%-09%-14$"), grid.row(2, 29))
  end,

  ["a narrow winbar keeps the view and the project and cuts the counts"] = function()
    opened()
    vim.cmd("vertical 20split")
    assert(vim.startswith(grid.row(1, 20), " default · app/"), grid.row(1, 20))
  end,

  ["a global view shows all in place of the project and reads no config"] = function()
    local fake = opened(nil, nil, { views = { everywhere = { global = true } } })
    assert(grid.row(1):find(" everywhere · all · 6 tasks", 1, true), grid.row(1))
    assert(#fake.of("config") == 0, vim.inspect(fake.of("config")))
  end,

  ["a row with nothing on the right that passes the window edge ends in an ellipsis"] = function()
    local answer = fixtures.decoded("list-columns.json")
    answer.objects[4].subject = ("a call that runs on and on "):rep(4)
    opened({ code = 0, stdout = vim.json.encode(answer), stderr = "" })
    local row = rows_on_screen()[5]
    assert(row:find("runs on…$"), row)
  end,
}
