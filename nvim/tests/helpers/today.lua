local buffer = require("helpers.buffer")
local fixtures = require("helpers.fixtures")
local helper = require("helpers.app")

local M = {}

M.ANSWERS = {
  ['{"overdue":true}'] = "today-overdue.json",
  ['{"due":"today","overdue":false}'] = "today-today.json",
  ['{"due":"today+1..today+7","overdue":false}'] = "today-next.json",
  ['{"due":"none"}'] = "today-none.json",
}

---@param stdin string
---@return string
local function canonical(stdin)
  local request = vim.json.decode(stdin)
  local keys = vim.tbl_keys(request)
  table.sort(keys)
  return "{"
    .. table.concat(
      vim.tbl_map(function(key)
        return vim.json.encode(key) .. ":" .. vim.json.encode(request[key])
      end, keys),
      ","
    )
    .. "}"
end

---@param answer string
---@param subject? string
---@return table reply
local function answered(answer, subject)
  if not subject then
    return fixtures.answer(answer)
  end
  local decoded = fixtures.decoded(answer)
  decoded.objects[1].subject = subject
  return { code = 0, stdout = vim.json.encode(decoded), stderr = "" }
end

---@param refused? table<string, boolean> the requests dam refuses
---@param subjects? table<string, string> a new subject for the first row of a request's answer
---@param held? boolean answer nothing until the spec releases each call
---@return damnit.App app, table fake
function M.opened(refused, subjects, held)
  local app, fake = helper.new()
  require("damnit.command").define(app, { require("damnit.list").new(app) })
  fake.reply("list", function(_, opts)
    local request = canonical(opts.stdin)
    if (refused or {})[request] then
      return fixtures.refusal("error-usage-selection.json", 2)
    end
    local reply = answered(assert(M.ANSWERS[request], "no today answer for " .. request), (subjects or {})[request])
    return vim.tbl_extend("force", reply, { hold = held })
  end)
  vim.cmd("Dam today")
  if held then
    return app, fake
  end
  buffer.wait_for(function()
    return #buffer.lines() > 4
  end)
  return app, fake
end

---@param text string
function M.on(text)
  local row = vim.fn.index(buffer.lines(), text) + 1
  assert(row > 0, ("no line %q in %s"):format(text, vim.inspect(buffer.lines())))
  vim.api.nvim_win_set_cursor(0, { row, 0 })
end

return M
