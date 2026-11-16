local actions = require("damnit.status.actions")
local confirm = require("damnit.kernel.confirm")
local confirm_lines = require("damnit.kernel.confirm_lines")

local Confirms = {}

---@param title string
---@param lines string[]
---@param marks damnit.Mark[]
---@param yes fun()
function Confirms:ask(title, lines, marks, yes)
  confirm.open(self.app.screen, { title = title, lines = lines, marks = marks, yes = yes })
end

---@param target damnit.StatusTarget
---@param argv string[]
function Confirms:decline(target, argv)
  local rows, summary = actions.declined(self.answer, target)
  local lines, marks = confirm_lines.rows(rows, summary, true)
  self:ask("dam rm", lines, marks, function()
    self:send(argv)
  end)
end

---@param args string[] the call to make after a yes
---@param read fun(answer: table): string[], damnit.Mark[]
---@param yes fun()
function Confirms:previewed(args, read, yes)
  local preview = vim.list_extend(vim.list_slice(args), { "--preview" })
  self.app.dam:run({ args = preview, cwd = self.cwd }, function(answer, err)
    if err then
      self.app.say:keep(err)
      return self:refused(err)
    end
    local lines, marks = read(answer)
    self:ask("dam " .. args[1], lines, marks, yes)
  end)
end

---@param argv string[]
function Confirms:restore(argv)
  self:previewed(argv, confirm_lines.changes, function()
    self:send(argv)
  end)
end

function Confirms:push()
  self:previewed({ "push" }, confirm_lines.push, function()
    self.writes.push(self.cwd)
  end)
end

return Confirms
