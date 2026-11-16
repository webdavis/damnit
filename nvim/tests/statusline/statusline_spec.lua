local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")
local status = require("helpers.status")

local EXIT_REFUSED = 4
local SHOWN = "!1 ?2 M1 S1 ^2"

local function defined()
  pcall(vim.api.nvim_del_augroup_by_name, "damnit.refresh")
  local app, fake = helper.new()
  fake.reply("list", fixtures.answer("list-overdue.json"))
  fake.reply("status", fixtures.answer("status.json"))
  return require("damnit.statusline").new(app), fake
end

local function reads(fake, count)
  buffer.wait_for(function()
    return #fake.of("status") >= count
  end)
  vim.wait(20)
  assert(#fake.of("status") == count, ("%d status reads, want %d"):format(#fake.of("status"), count))
end

local function first_read(statusline, fake)
  statusline.component()
  reads(fake, 1)
  buffer.wait_for(function()
    return statusline.component() == SHOWN
  end)
end

return {
  ["the statusline never starts dam: a hundred asks return at once and start only the first read"] = function()
    local statusline, fake = defined()
    for _ = 1, 100 do
      assert(statusline.component() == "")
    end
    assert(#fake.calls == 0, vim.inspect(fake.calls))
    reads(fake, 1)
    assert(#fake.of("list") == 1)
  end,

  ["counts read from dam appear"] = function()
    local statusline, fake = defined()
    local cwd = status.folder()
    first_read(statusline, fake)
    local list = fake.of("list")[1]
    assert(
      vim.deep_equal(list.argv, { "dam", "list", "--overdue", "-o", "json" }) and list.cwd == cwd,
      vim.inspect(list)
    )
    assert(fake.of("status")[1].cwd == cwd)
    local want = { overdue = 1, waiting = 2, unstaged = 1, staged = 1, unpushed = 2 }
    assert(vim.deep_equal(statusline.counts(), want), vim.inspect(statusline.counts()))
    assert(statusline.component({ highlight = true }):find("%#DamOverdue#!1%*", 1, true))
  end,

  ["a failed read keeps the last counts"] = function()
    local statusline, fake = defined()
    first_read(statusline, fake)
    local notes = buffer.capture_notes()
    fake.reply("status", fixtures.refusal("error-refused.json", EXIT_REFUSED))
    vim.api.nvim_exec_autocmds("User", { pattern = "DamChanged", data = { cwd = vim.fn.getcwd() } })
    reads(fake, 2)
    vim.wait(20)
    assert(statusline.component() == SHOWN, statusline.component())
    assert(#notes == 0, vim.inspect(notes))
  end,

  ["DamChanged, FocusGained and DirChanged each start one read"] = function()
    local statusline, fake = defined()
    first_read(statusline, fake)
    vim.api.nvim_exec_autocmds("User", { pattern = "DamChanged", data = { cwd = "/elsewhere" } })
    reads(fake, 2)
    vim.api.nvim_exec_autocmds("FocusGained", {})
    reads(fake, 3)
    local folder = status.folder()
    reads(fake, 4)
    assert(fake.of("list")[4].cwd == folder, fake.of("list")[4].cwd .. " is not " .. folder)
  end,

  ["before the first answer there are no counts"] = function()
    local statusline = defined()
    assert(statusline.counts() == nil)
  end,
}
