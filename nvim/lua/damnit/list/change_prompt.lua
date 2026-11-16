local change_lines = require("damnit.list.change_lines")
local words = require("damnit.kernel.words")

local M = {}

local function on_screen(list)
  local objects, rows = {}, {}
  for _, row in ipairs(list.arranged or {}) do
    objects[row.object.oid] = row.object
    table.insert(rows, { word = row.object.oid:sub(1, 7), menu = row.object.subject })
  end
  return objects, rows
end

local function judged(answer, objects, prompt, many)
  if not many then
    return change_lines.reading(answer, objects, prompt.field), false, nil
  end
  local summary = ("%d of %d would change"):format(#answer.changes, answer.count)
  return summary, false, change_lines.rows(answer, objects, prompt.field)
end

---@param lists table the list windows, with their edits
---@param buf integer
---@param key string
---@param oids string[]
---@param prompt { title: string, prefill: string, field?: string }
---@param submit fun(text: string)
function M.open(lists, buf, key, oids, prompt, submit)
  local app, list = lists.app, lists.shown[buf]
  local objects, rows = on_screen(list)
  local flags = app.vocab:flags(vim.split(prompt.title, " ")[2])
  lists.prompts = lists.prompts or require("damnit.kernel.prompt").new(app.queue)
  lists.prompts:open({
    title = prompt.title .. (#oids > 1 and (" · %d rows"):format(#oids) or ""),
    prefill = prompt.prefill,
    anchor = vim.api.nvim_get_current_win(),
    flags = flags,
    read = function(text, done)
      return app.dam:run(lists:call_of(buf, key, oids, text, true), function(answer, err)
        if err then
          return done(err.message, true)
        end
        done(judged(answer, objects, prompt, #oids > 1))
      end)
    end,
    submit = submit,
    complete = function(line, col)
      local vocab = vim.tbl_extend("force", {}, app.vocab:words(list.call.cwd), { rows = rows })
      return words.at(line, col, flags, vocab)
    end,
  })
  app.vocab:words(list.call.cwd)
end

return M
