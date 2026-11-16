local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

return {
  [":Dam status opens dam://status with dam's sections"] = function()
    local _, fake = status.opened()
    assert(vim.api.nvim_buf_get_name(0) == "dam://status", vim.api.nvim_buf_get_name(0))
    assert(vim.bo.filetype == "damstatus", vim.bo.filetype)
    assert(vim.deep_equal(fake.of("status")[1].argv, { "dam", "status", "--full", "-o", "json" }))
    assert(buffer.lines()[1] == "Conflicts (1)", vim.inspect(buffer.lines()))
  end,

  [":Dam alone opens the status window"] = function()
    local _, fake = status.defined()
    status.open(fake, "Dam")
    assert(vim.api.nvim_buf_get_name(0) == "dam://status", vim.api.nvim_buf_get_name(0))
  end,

  ["the status window runs dam in the folder it was opened in"] = function()
    local _, fake = status.defined()
    local opened_in = status.folder()
    status.open(fake)
    status.folder()
    status.on("  update  call mom back")
    buffer.feed("s")
    status.sent(fake, "add")
    buffer.feed("r")
    buffer.wait_for(function()
      return #fake.of("status") >= 2
    end)
    for _, call in ipairs(vim.list_extend(fake.of("status"), fake.of("add"))) do
      assert(call.cwd == opened_in, call.cwd)
    end
  end,

  ["DamChanged rereads it"] = function()
    local app, fake = status.opened()
    fake.reply("status", fixtures.answer("status-empty.json"))
    app.say:changed("/another/folder")
    buffer.wait_for(function()
      return buffer.lines()[1] == "nothing changed, staged, waiting or unpushed"
    end)
  end,

  ["a change sent from the window reads it again"] = function()
    local _, fake = status.opened()
    fake.reply("status", fixtures.answer("status-empty.json"))
    status.on("  update  call mom back")
    buffer.feed("s")
    buffer.wait_for(function()
      return #buffer.lines() == 1
    end)
  end,

  ["a wiped status window is not read again"] = function()
    local app, fake = status.opened()
    vim.cmd.bwipeout({ bang = true })
    app.say:changed("/")
    vim.wait(30)
    assert(#fake.of("status") == 1, vim.inspect(fake.of("status")))
  end,

  ["the status window's winbar counts each section"] = function()
    status.opened()
    local row = require("helpers.grid").row(1)
    assert(
      vim.startswith(row, " status · 1 conflict · 2 waiting · 1 unstaged · 1 staged · 2 unpushed commits"),
      row
    )
  end,

  ["a long change row does not wrap: its subject ends in an ellipsis and its fields end at the last column"] = function()
    local _, fake = status.defined()
    local answer = fixtures.decoded("status.json")
    answer.unstaged[1].subject = ("call mom back about the long weekend "):rep(3)
    fake.reply("status", { code = 0, stdout = vim.json.encode(answer), stderr = "" })
    status.open(fake)
    local grid = require("helpers.grid")
    local row = vim.fn.index(buffer.lines(), "Unstaged (1)") + 2
    assert(vim.wo.wrap == false)
    assert(grid.row(row + 1):find("the long weekend call mom back about the … subject$"), grid.row(row + 1))
    assert(grid.row(row + 2) == (" "):rep(vim.o.columns), grid.row(row + 2))
  end,

  ["a narrow status window keeps its name in the winbar"] = function()
    status.opened()
    vim.cmd("vertical 12split")
    assert(vim.startswith(require("helpers.grid").row(1, 12), " status"), require("helpers.grid").row(1, 12))
  end,

  ["a subject too long for a narrow status window ends in an ellipsis at its edge"] = function()
    local _, fake = status.defined()
    local answer = fixtures.decoded("status.json")
    answer.unstaged[1].subject = ("call mom back about the long weekend "):rep(3)
    answer.unstaged[1].path = ("garden/plan the offsite/"):rep(2)
    fake.reply("status", { code = 0, stdout = vim.json.encode(answer), stderr = "" })
    status.open(fake)
    vim.cmd("vertical 40split")
    vim.api.nvim_exec_autocmds("WinResized", {})
    local row = vim.fn.index(buffer.lines(), "Unstaged (1)") + 2
    buffer.wait_for(function()
      return require("helpers.grid").row(row + 1, 40):find("…$") ~= nil
    end)
    local shown = require("helpers.grid").row(row + 1, 40)
    assert(shown:find("^  update  call mom back about the long.*…$"), shown)
  end,
}
