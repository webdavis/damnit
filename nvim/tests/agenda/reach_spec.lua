local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local status = require("helpers.status")

local function defined()
  local app, fake = status.defined()
  local list = require("damnit.list").new(app)
  require("damnit.command").define(app, {
    require("damnit.status").new(app, function() end),
    list,
    require("damnit.agenda").new(app, list.show),
    require("damnit.plan").new(app, list.show),
  })
  fake.reply("list", fixtures.answer("plan-empty.json"))
  fake.reply("agenda", fixtures.answer("agenda-week.json"))
  return app, fake
end

local function opened(name)
  buffer.wait_for(function()
    return vim.api.nvim_buf_get_name(0) == name
  end)
end

return {
  ["ga in a list opens this week's agenda"] = function()
    local _, fake = defined()
    vim.cmd("Dam list")
    opened("dam://list/default")
    buffer.feed("ga")
    opened("dam://agenda")
    buffer.wait_for(function()
      return #fake.of("agenda") == 1
    end)
  end,

  ["gp in the status window opens the week plan"] = function()
    local _, fake = defined()
    status.open(fake)
    buffer.feed("gp")
    opened("dam://plan")
  end,

  ["ga in the week plan opens the agenda and gp there opens the plan again"] = function()
    defined()
    vim.cmd("Dam plan")
    opened("dam://plan")
    buffer.feed("ga")
    opened("dam://agenda")
    buffer.feed("gp")
    opened("dam://plan")
  end,

  ["<Plug>(dam-agenda) runs :Dam agenda"] = function()
    dofile(vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "../../plugin/damnit.lua"))
    local mapped = vim.fn.maparg("<Plug>(dam-agenda)", "n")
    assert(mapped == "<Cmd>Dam agenda<CR>", mapped)
  end,

  ["<Plug>(dam-plan) runs :Dam plan"] = function()
    dofile(vim.fs.joinpath(vim.fs.dirname(debug.getinfo(1, "S").source:sub(2)), "../../plugin/damnit.lua"))
    local mapped = vim.fn.maparg("<Plug>(dam-plan)", "n")
    assert(mapped == "<Cmd>Dam plan<CR>", mapped)
  end,
}
