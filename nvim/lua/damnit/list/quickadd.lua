local new_line = require("damnit.list.new_line")
local words = require("damnit.kernel.words")

local M = {}

---@class damnit.QuickAdd
---@field open fun(cwd: string, after: string, landing?: damnit.Landing, anchor?: integer)
---@field send fun(text: string, cwd: string, landing?: damnit.Landing)

---@alias damnit.Landing fun(oid: string, done: fun(found: boolean))

---@param app damnit.App
---@param prompts damnit.Prompts
---@return damnit.QuickAdd
function M.new(app, prompts)
  local quickadd = {}

  local function told(answer, landing)
    if answer.hash then
      return app.say:info("sent for the owner: " .. answer.hash)
    end
    local said = "new: " .. answer.subject
    if not landing then
      return app.say:info(said)
    end
    landing(answer.oid, function(found)
      app.say:info(found and said or (said .. "; not in this view"))
    end)
  end

  function quickadd.send(text, cwd, landing)
    app.dam:write({ args = new_line.args(text, app.vocab:flags("new")), cwd = cwd }, function(answer)
      if answer then
        told(answer, landing)
      end
    end)
  end

  local function read(cwd)
    return function(text, done)
      local args = vim.list_extend(new_line.args(text, app.vocab:flags("new")), { "--preview" })
      return app.dam:run({ args = args, cwd = cwd }, function(answer, err)
        if err then
          return done(err.message, err)
        end
        done(new_line.reading(answer), false)
      end)
    end
  end

  function quickadd.open(cwd, after, landing, anchor)
    prompts:open({
      title = "dam new",
      anchor = anchor,
      prefill = "",
      after = after,
      flags = app.vocab:flags("new"),
      read = read(cwd),
      submit = function(text)
        quickadd.send(text, cwd, landing)
      end,
      complete = function(line, col)
        return words.at(line, col, app.vocab:flags("new"), app.vocab:words(cwd))
      end,
    })
    app.vocab:words(cwd)
  end

  return quickadd
end

return M
