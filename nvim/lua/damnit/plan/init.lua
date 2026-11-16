local anchors = require("damnit.plan.anchors")

local M = {}

local SECTIONS = anchors.sections()

---@param answers { answer?: damnit.ListAnswer, err?: damnit.Error }[]
---@return damnit.Segment[]
local function segments(answers)
  return vim.tbl_map(function(n)
    local section, got = SECTIONS[n], answers[n]
    if got.err then
      return { title = section.title, error = got.err }
    end
    local objects = got.answer.objects
    return { title = anchors.header(section.title, objects), key = section.title, uncounted = true, objects = objects }
  end, vim.fn.range(1, #SECTIONS))
end

---@param say damnit.Say
---@param window damnit.ListWindow
---@param step 1|-1
local function move(say, window, step)
  local oids = window.targets()
  if #oids == 0 then
    return say:warn("> and < move task rows; this line holds no task")
  end
  local title = anchors.section_of(window.rows(), oids)
  if not title then
    return say:warn("> and < move the rows of one section at a time")
  end
  local flags, problem = anchors.moved(title, step)
  if not flags then
    return say:warn(problem)
  end
  window.edit(oids, flags)
end

---@param say damnit.Say
---@return fun(window: damnit.ListWindow): damnit.KeySpec[]
local function keys(say)
  return function(window)
    local function key(lhs, step, desc)
      return {
        lhs = lhs,
        desc = desc,
        group = "week plan",
        mode = { "n", "x" },
        run = function()
          move(say, window, step)
        end,
      }
    end
    return { key(">", 1, "plan a day later"), key("<", -1, "plan a day earlier, or not at all from today") }
  end
end

---@param app damnit.App
---@param show fun(source: damnit.ListSource, cwd: string, smods?: table)
---@return damnit.Feature
function M.new(app, show)
  local source = { name = "plan", view = SECTIONS, segments = segments, keys = keys(app.say) }
  return {
    commands = {
      plan = function(_, cmd)
        show(source, vim.fn.getcwd(), cmd.smods)
      end,
    },
  }
end

return M
