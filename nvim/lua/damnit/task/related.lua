local M = {}

local NAME = "dam://task/"

---@return string|nil
local function oid_under_cursor()
  return vim.fn.expand("<cword>"):match("^%x%x%x%x%x%x%x+$")
end

---@param label string
---@param oid? string the oid asked for, else the object's own
---@return fun(object: damnit.Object): vim.quickfix.entry
local function entry(label, oid)
  return function(object)
    return { filename = NAME .. (oid or object.oid), lnum = 1, text = label .. ": " .. object.subject }
  end
end

---@param app damnit.App
---@param win integer
---@param slots vim.quickfix.entry[][]
---@param count integer
local function listed(app, win, slots, count)
  local items = {}
  for n = 1, count do
    vim.list_extend(items, slots[n] or {})
  end
  if #items == 0 then
    return app.say:info("this task has no parent, children or dependencies")
  end
  if not vim.api.nvim_win_is_valid(win) then
    return
  end
  vim.fn.setloclist(win, {}, " ", { title = "related tasks", items = items })
  vim.api.nvim_win_call(win, vim.cmd.lopen)
end

---@param app damnit.App
---@param object damnit.Object
---@param cwd string
---@param win integer
local function gather(app, object, cwd, win)
  local slots, asked = {}, {}
  local function ask(n, call, read)
    table.insert(asked, { n = n, call = call, read = read })
  end
  if object.parent then
    ask(1, { args = { "show", object.parent }, cwd = cwd }, function(answer)
      return { entry("parent", object.parent)(answer) }
    end)
  end
  local children = { args = { "list", "--request", "-" }, stdin = vim.json.encode({ parent = object.oid }), cwd = cwd }
  ask(2, children, function(answer)
    return vim.tbl_map(entry("child"), answer.objects)
  end)
  for n, oid in ipairs(object.depends or {}) do
    ask(2 + n, { args = { "show", oid }, cwd = cwd }, function(answer)
      return { entry("waits on", oid)(answer) }
    end)
  end
  local pending = #asked
  for _, question in ipairs(asked) do
    app.dam:run(question.call, function(answer, err)
      if err then
        app.say:error(err)
      end
      slots[question.n] = answer and question.read(answer) or {}
      pending = pending - 1
      if pending == 0 then
        listed(app, win, slots, 2 + #(object.depends or {}))
      end
    end)
  end
end

---@param app damnit.App
---@param tasks table
---@param buf integer
---@param peek? damnit.Peek
---@return damnit.KeySpec[]
local function keys(app, tasks, buf, peek)
  local function task()
    return tasks:task_of(buf)
  end
  local specs = {
    {
      lhs = "gO",
      desc = "list the parent, children and dependencies",
      group = "task buffer",
      mode = "n",
      run = function()
        local win = vim.api.nvim_get_current_win()
        app.dam:run({ args = { "show", task().oid }, cwd = task().cwd }, function(object, err)
          if err then
            return app.say:error(err)
          end
          gather(app, object, task().cwd, win)
        end)
      end,
    },
    {
      lhs = "gd",
      desc = "open the task whose oid is under the cursor",
      group = "task buffer",
      mode = "n",
      run = function()
        local oid = oid_under_cursor()
        if not oid then
          return app.say:warn("no oid under the cursor to open")
        end
        tasks:open(oid, task().cwd)
      end,
    },
  }
  if peek then
    table.insert(
      specs,
      peek.key(function()
        return oid_under_cursor(), task().cwd
      end, "task buffer", "no oid under the cursor to peek at")
    )
  end
  return specs
end

---@param app damnit.App
---@param tasks table the task buffers: task_of(buf) and open(oid, cwd)
---@param peek? damnit.Peek
function M.attach(app, tasks, peek)
  vim.api.nvim_create_autocmd("FileType", {
    group = vim.api.nvim_create_augroup("damnit.task.related", { clear = true }),
    pattern = "damtask",
    callback = function(args)
      app.keys:bind(args.buf, keys(app, tasks, args.buf, peek))
    end,
  })
end

return M
