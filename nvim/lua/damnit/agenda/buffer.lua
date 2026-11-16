local days = require("damnit.agenda.days")

local M = {}

---@alias damnit.Answers { answer?: table, err?: damnit.Error }[]

---@param window string
---@param holds_today boolean
---@return damnit.ListPart[]
local function parts(window, holds_today)
  local agenda = { args = { "agenda", "--kind", "any", "--from", window, "--to", window } }
  if not holds_today then
    return { agenda }
  end
  return { { title = "Overdue", args = { "list", "--overdue" } }, agenda }
end

---@param answers damnit.Answers
---@return damnit.Segment[]|nil segments, damnit.Error|nil err
local function segments(answers)
  local agenda = answers[#answers]
  if agenda.err then
    return nil, agenda.err
  end
  local overdue = #answers > 1 and answers[1] or nil
  local found = {}
  if overdue then
    found[1] = { title = "Overdue", objects = overdue.answer and overdue.answer.objects, error = overdue.err }
  end
  return vim.list_extend(found, days.segments(agenda.answer)), nil
end

---@param answers damnit.Answers
---@param marked integer
---@return string[][]
local function bar(answers, marked)
  return days.bar(answers[#answers].answer, marked)
end

---@class damnit.Agenda
---@field open fun(unit: "day"|"week", offset: integer, cwd: string, smods?: table)

---@param show fun(source: damnit.ListSource, cwd: string, smods?: table)
---@return damnit.Agenda
function M.new(show)
  local agenda, at = {}, {}
  local function moved(offset)
    return function()
      agenda.open(at.unit, offset(at.offset), at.cwd)
    end
  end
  local keys = {
    { lhs = ">", desc = "the next day or week", group = "agenda", run = moved(function(n)
      return n + 1
    end) },
    {
      lhs = "<",
      desc = "the day or week before",
      group = "agenda",
      run = moved(function(n)
        return n - 1
      end),
    },
    {
      lhs = "T",
      desc = "back to today or this week",
      group = "agenda",
      run = moved(function()
        return 0
      end),
    },
  }
  function agenda.open(unit, offset, cwd, smods)
    at.unit, at.offset, at.cwd = unit, offset, cwd
    show({
      name = "agenda",
      filetype = "damagenda",
      parts = parts(days.window(unit, offset), offset == 0),
      segments = segments,
      bar = bar,
      keys = function()
        return keys
      end,
    }, cwd, smods)
  end
  return agenda
end

return M
