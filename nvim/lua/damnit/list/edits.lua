local actions = require("damnit.list.actions")
local batch = require("damnit.kernel.batch")
local change_prompt = require("damnit.list.change_prompt")
local confirm = require("damnit.kernel.confirm")
local confirm_lines = require("damnit.kernel.confirm_lines")
local diagnostics = require("damnit.kernel.diagnostics")

local Edits = {}

---@param key string
---@param oids string[]
---@param answer? string
---@param preview? boolean
---@return damnit.Call
function Edits:call_of(buf, key, oids, answer, preview)
  local spec = actions.batch(key, oids, answer, function(command)
    return self.app.vocab:flags(command)
  end)
  spec.preview, spec.token = preview, vim.env.DAM_TOKEN ~= nil
  local args, stdin = batch.call(spec)
  return { args = args, stdin = stdin, cwd = self.shown[buf].call.cwd, placed = true }
end

---@param buf integer
---@param err damnit.Error
function Edits:refused(buf, err)
  if vim.api.nvim_buf_is_valid(buf) and self.shown[buf] then
    diagnostics.show(buf, err, diagnostics.rows_of(err, self.shown[buf].oids))
  end
end

---@param buf integer
---@param key string
---@param oids string[]
---@param answer? string
function Edits:send(buf, key, oids, answer)
  local row = vim.fn.bufnr() == buf and vim.api.nvim_win_get_cursor(0)[1] or 1
  local oid = self.shown[buf].oids[row]
  self.app.dam:write(self:call_of(buf, key, oids, answer), function(written, err)
    if err then
      return self:refused(buf, err)
    end
    diagnostics.clear(buf)
    if self.shown[buf] then
      self:keep_cursor(buf, oid, row)
    end
    local held = written.hash or written.set
    self.app.say:info(held and ("sent for the owner: " .. held) or actions.said(key, #oids))
  end)
end

local function shown_changes(preview, count)
  if preview.changes then
    return preview, nil
  end
  return { count = count, changes = preview.staged }, "would be staged"
end

---@param buf integer
---@param key string
---@param oids string[]
---@param answer? string
function Edits:confirm(buf, key, oids, answer)
  local call = self:call_of(buf, key, oids, answer, true)
  self.app.dam:run(call, function(preview, err)
    if err then
      self.app.say:keep(err)
      return self:refused(buf, err)
    end
    local lines, marks = confirm_lines.changes(shown_changes(preview, #oids))
    confirm.open(self.app.screen, {
      title = "dam " .. call.args[1],
      lines = lines,
      marks = marks,
      yes = function()
        self:send(buf, key, oids, answer)
      end,
    })
  end)
end

---@param buf integer
---@param key string
---@param oids string[]
---@param answer? string
---@param remember? fun(answer: string)
function Edits:act(buf, key, oids, answer, remember)
  local prompt = actions.prompt(key, self:object_of(buf, oids[1]))
  if prompt and prompt.one and #oids > 1 then
    return self.app.say:warn(key .. " changes one row at a time")
  end
  if prompt and not answer then
    return change_prompt.open(self, buf, key, oids, prompt, function(text)
      if remember then
        remember(text)
      end
      self:send(buf, key, oids, text)
    end)
  end
  if actions.confirms(key, oids) then
    return self:confirm(buf, key, oids, answer)
  end
  self:send(buf, key, oids, answer)
end

---@param buf integer
---@param oid string
---@return damnit.Object|nil
function Edits:object_of(buf, oid)
  for _, row in ipairs(self.shown[buf].arranged or {}) do
    if row.object and row.object.oid == oid then
      return row.object
    end
  end
end

return Edits
