local new_line = require("damnit.capture.new_line")
local text = require("damnit.capture.text")
local words = require("damnit.kernel.words")

local M = {}

---@class damnit.Capture
---@field subject string
---@field place? string
---@field cwd string
---@field line integer

---@param cmd table
---@return damnit.Capture
local function captured(cmd)
  local buf = vim.api.nvim_get_current_buf()
  local file = vim.bo[buf].buftype == "" and vim.api.nvim_buf_get_name(buf) or ""
  local folder = file ~= "" and vim.fs.dirname(file) or ""
  local cwd = vim.fn.isdirectory(folder) == 1 and folder or vim.fn.getcwd()
  local lines = vim.api.nvim_buf_get_lines(buf, cmd.line1 - 1, cmd.line2, false)
  local shape = { commentstring = vim.bo[buf].commentstring, comments = vim.bo[buf].comments }
  return { subject = text.subject(lines, shape), place = text.place(file, cwd, cmd.line1), cwd = cwd, line = cmd.line1 }
end

---@param app damnit.App
---@return damnit.Feature
function M.new(app)
  local prompts = require("damnit.kernel.prompt").new(app.queue)

  local function send(args, cwd)
    app.dam:write({ args = args, cwd = cwd }, function(answer)
      if answer then
        app.say:info(answer.hash and ("sent for the owner: " .. answer.hash) or ("captured " .. answer.subject))
      end
    end)
  end

  local function read(cwd)
    return function(line, done)
      local args = vim.list_extend(new_line.args(line), { "--preview" })
      return app.dam:run({ args = args, cwd = cwd }, function(answer, err)
        if err then
          return done(err.message, true)
        end
        done(new_line.reading(answer), false)
      end)
    end
  end

  local function opened(capture)
    prompts:open({
      title = "dam new",
      read_first = true,
      anchor = "cursor",
      prefill = text.typed(capture.subject),
      after = capture.place and (" --body " .. text.quoted(capture.place)) or "",
      read = read(capture.cwd),
      submit = function(line)
        send(new_line.args(line), capture.cwd)
      end,
      complete = function(line, col)
        return words.at(line, col, app.vocab:flags("new"), app.vocab:words(capture.cwd))
      end,
    })
    app.vocab:words(capture.cwd)
  end

  return {
    commands = {
      capture = function(_, cmd)
        local capture = captured(cmd)
        if capture.subject == "" then
          return app.say:warn(("nothing to capture on line %d"):format(capture.line))
        end
        if not cmd.bang then
          vim.api.nvim_win_set_cursor(0, { cmd.line2, 0 })
          return opened(capture)
        end
        send(
          vim.list_extend({ "new", capture.subject }, capture.place and { "--body", capture.place } or {}),
          capture.cwd
        )
      end,
    },
  }
end

return M
