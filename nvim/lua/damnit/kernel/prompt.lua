local float = require("damnit.kernel.float")
local judge = require("damnit.kernel.judge")
local prompt_keys = require("damnit.kernel.prompt_keys")
local prompt_session = require("damnit.kernel.prompt_session")

local Prompts = {}
Prompts.__index = Prompts

local M = {}

---@class damnit.PromptSpec
---@field title string
---@field prefill string
---@field after? string
---@field read fun(text: string, done: fun(answer: string, refused: boolean|damnit.Error, below?: string[])): damnit.Handle
---@field submit fun(text: string)
---@field complete? fun(line: string, col: integer): integer, damnit.Candidate[]
---@field anchor? damnit.Anchor
---@field read_first? boolean
---@field closed? fun(sent: boolean) runs once the prompt closes
---@field reads_blank? boolean an empty line is read and sent like any other
---@field keys? table<string, fun(session: damnit.PromptSession)> more insert-mode keys
---@field flags? damnit.Flag[] the command's flags; a word at the cursor that begins one is not read yet

---@class damnit.Prompts
---@field open fun(self, spec: damnit.PromptSpec)

---@param queue damnit.Queue
---@param say damnit.Say keeps the refusal of a line sent with <CR> for :Dam messages
---@return damnit.Prompts
function M.new(queue, say)
  return setmetatable({ queue = queue, say = say, histories = {} }, Prompts)
end

local function completion(buf, complete)
  local found = {}
  vim.b[buf].damnit_complete = function(findstart)
    if findstart == 1 then
      local start, candidates = complete(vim.api.nvim_get_current_line(), vim.fn.col(".") - 1)
      found = candidates
      return start
    end
    return found
  end
  vim.api.nvim_buf_call(buf, function()
    vim.cmd("let &l:completefunc = b:damnit_complete")
  end)
  vim.bo[buf].complete = "F"
  vim.bo[buf].completeopt = "menuone,noselect,popup"
  vim.bo[buf].autocomplete = true
end

local function watched(session)
  vim.api.nvim_buf_attach(session.buf, false, {
    on_lines = function()
      if not session.prefilling then
        vim.schedule(function()
          session:changed()
        end)
      end
    end,
  })
end

---@param spec damnit.PromptSpec
function Prompts:open(spec)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].bufhidden = "wipe"
  local win = float.input(buf, spec.title, spec.anchor)
  vim.bo[buf].filetype = "damprompt"
  self.histories[spec.title] = self.histories[spec.title] or {}
  local show = function(answer, refused, below)
    float.footer(win, answer, refused and "DiagnosticError" or "FloatFooter", below)
  end
  local session = prompt_session.new({
    win = win,
    buf = buf,
    spec = spec,
    say = self.say,
    entries = self.histories[spec.title],
    judge = judge.new(self.queue, "prompt " .. buf, spec.read, show),
  })
  if spec.complete then
    completion(buf, spec.complete)
  end
  prompt_keys.bind(session)
  watched(session)
  session:prefill()
  if spec.read_first then
    session:changed()
  end
  vim.cmd(spec.after and spec.after ~= "" and "startinsert" or "startinsert!")
end

return M
