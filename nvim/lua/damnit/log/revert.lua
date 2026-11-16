local confirm = require("damnit.kernel.confirm")
local confirm_lines = require("damnit.kernel.confirm_lines")
local diagnostics = require("damnit.kernel.diagnostics")
local reverted = require("damnit.log.reverted")

local Revert = {}

---@param commit damnit.Commit
---@return string[] lines, damnit.Mark[] marks
local function confirm_of(commit)
  local count = #commit.changes
  local rows = vim.tbl_map(function(change)
    return { op = change.op, subject = change.subject, detail = table.concat(change.fields, ", ") }
  end, commit.changes)
  local summary = count == 1 and "1 change comes back as a working change"
    or ("%d changes come back as working changes"):format(count)
  return confirm_lines.rows(rows, summary, false)
end

---@param id string the commit under the cursor
---@param commit damnit.Commit as dam show printed it
---@param line integer the commit's row, counted from 1
function Revert:send_revert(id, commit, line)
  local buf = self.buf
  self.app.dam:write({ args = { "revert", id }, cwd = self.cwd, placed = true }, function(answer, err)
    if not (buf and vim.api.nvim_buf_is_valid(buf)) then
      return
    end
    if err then
      return diagnostics.show(buf, err, { line - 1 })
    end
    diagnostics.clear(buf)
    self.app.say:info(reverted.text(answer, commit))
    self.open_status(self.cwd)
  end)
end

function Revert:revert()
  local commit, line = self:commit()
  if not commit then
    return self.app.say:warn("R reverts the commit under the cursor")
  end
  self.app.dam:run({ args = { "show", commit.id }, cwd = self.cwd }, function(shown, err)
    if err then
      return self.app.say:error(err)
    end
    local lines, marks = confirm_of(shown)
    confirm.open(self.app.screen, {
      title = "dam revert " .. commit.id:sub(1, 7),
      lines = lines,
      marks = marks,
      yes = function()
        self:send_revert(commit.id, shown, line)
      end,
    })
  end)
end

return Revert
