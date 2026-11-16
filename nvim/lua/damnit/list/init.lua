local buffer = require("damnit.list.buffer")
local new_keys = require("damnit.list.new_keys")
local views = require("damnit.list.view")

local M = {}

---@param app damnit.App
---@return string[]
local function filter_names(app)
  local names
  app.dam:run({ args = { "filter", "list" }, cwd = vim.fn.getcwd(), key = "filter names" }, function(answer)
    names = answer and vim.tbl_map(function(filter)
      return filter.name
    end, answer.filters) or {}
  end)
  vim.wait(app.options.timeout_ms, function()
    return names ~= nil
  end, 10)
  return names or {}
end

---@param app damnit.App
---@return fun(arglead: string): string[]
local function view_names(app)
  return function(arglead)
    local names = vim.list_extend(vim.tbl_keys(app.options.views), filter_names(app))
    names = vim.tbl_filter(function(name)
      return vim.startswith(name, arglead)
    end, vim.fn.uniq(vim.fn.sort(names)))
    return names
  end
end

---@param app damnit.App
---@param open_task fun(oid: string, cwd: string)
---@return damnit.Feature
function M.new(app, open_task)
  local quickadd = require("damnit.list.quickadd").new(app, require("damnit.kernel.prompt").new(app.queue, app.say))
  local lists
  lists = buffer.new(
    app,
    open_task,
    new_keys.keys(app, quickadd, function()
      return lists
    end)
  )
  local picker = require("damnit.list.pick").new(app, lists, open_task)
  app.keys:everywhere({
    lhs = "go",
    desc = "pick a view to open",
    mode = "n",
    run = function()
      picker.views(vim.fn.getcwd())
    end,
  })
  return {
    open = function(view, cwd, smods)
      lists:open(view, cwd, smods)
    end,
    show = function(source, cwd, smods)
      lists:show(source, cwd, smods)
    end,
    commands = {
      list = function(args, cmd)
        lists:open(args[1], vim.fn.getcwd(), cmd.smods)
      end,
      new = new_keys.command(quickadd, lists),
      filter = function()
        local buf = vim.api.nvim_get_current_buf()
        if not lists.shown[buf] then
          return app.say:warn("the filter bar changes a list; open one with :Dam list first")
        end
        lists:filter(buf)
      end,
      pick = function(args, cmd)
        if args[1] then
          return picker.rows(args[1], vim.fn.getcwd())
        end
        picker.views(vim.fn.getcwd(), cmd.smods)
      end,
      today = function(_, cmd)
        lists:open("today", vim.fn.getcwd(), cmd.smods, views.TODAY)
      end,
    },
    complete = { list = view_names(app), pick = view_names(app) },
  }
end

return M
