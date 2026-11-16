local view = require("damnit.list.view")

local M = {}

---@class damnit.ViewChoice
---@field name string
---@field note string

---@class damnit.Pick
---@field views fun(cwd: string, smods?: table)
---@field rows fun(name: string, cwd: string)
---@field open fun(name: string|nil, cwd: string, smods?: table)

---@param app damnit.App
---@param name string
---@return string|damnit.ViewSpec[]
local function spec_of(app, name)
  if name == "today" and not app.options.views.today then
    return view.TODAY
  end
  return name
end

---@param app damnit.App
---@param cwd string
---@return damnit.ViewChoice[]
local function choices(app, cwd)
  local all = { { name = "today", note = "built in" } }
  local names = vim.tbl_keys(app.options.views)
  table.sort(names)
  for _, name in ipairs(names) do
    table.insert(all, { name = name, note = "view in your options" })
  end
  local words = app.vocab:words(cwd)
  vim.wait(app.options.timeout_ms, function()
    return words.filters ~= nil
  end, 10)
  for _, filter in ipairs(words.filters or {}) do
    table.insert(all, { name = filter.word, note = "saved filter" })
  end
  return all
end

---@param object damnit.Object
---@return string
local function row_text(object)
  local due = object.task and object.task.due or (object.event and object.event.start)
  local parts = vim.tbl_filter(function(part)
    return part ~= nil and part ~= ""
  end, { object.subject, due, object.path })
  return table.concat(parts, "  ")
end

---@param app damnit.App
---@param lists table the list buffers: open(name, cwd, smods, built_in)
---@param open_task fun(oid: string, cwd: string)
---@return damnit.Pick
function M.new(app, lists, open_task)
  local pick = {}

  function pick.open(name, cwd, smods)
    local spec = name and spec_of(app, name)
    lists:open(name, cwd, smods, type(spec) == "table" and spec or nil)
  end

  function pick.views(cwd, smods)
    vim.ui.select(choices(app, cwd), {
      prompt = "dam view",
      format_item = function(choice)
        return choice.name .. " · " .. choice.note
      end,
    }, function(choice)
      if choice then
        lists:open(choice.name, cwd, smods, choice.note == "built in" and view.TODAY or nil)
      end
    end)
  end

  function pick.rows(name, cwd)
    local parts = view.parts(spec_of(app, name), app.options.views)
    local answers = {}
    local function offer()
      local objects = {}
      for n = 1, #parts do
        vim.list_extend(objects, answers[n])
      end
      vim.ui.select(objects, { prompt = "dam " .. name, format_item = row_text }, function(object)
        if object then
          open_task(object.oid, cwd)
        end
      end)
    end
    for n, part in ipairs(parts) do
      app.dam:run({ args = part.args, stdin = part.stdin, cwd = cwd, key = "pick " .. n }, function(answer, err)
        if err then
          return app.say:error(err)
        end
        answers[n] = answer.objects
        if vim.tbl_count(answers) == #parts then
          offer()
        end
      end)
    end
  end

  return pick
end

return M
