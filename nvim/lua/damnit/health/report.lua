local version = require("damnit.kernel.version")

local LEVELS = { ok = "ok", warning = "warn", issue = "error", not_run = "info" }

local M = {}

---@class damnit.HealthItem
---@field level "ok"|"warn"|"error"|"info"
---@field text string
---@field advice? string

---@param found string|nil the version dam --version printed
---@param err damnit.Error|nil
---@return damnit.HealthItem
function M.dam(found, err)
  if not found then
    return { level = "error", text = err and err.message or "dam printed no version" }
  end
  if not version.supported(found) then
    return { level = "error", text = version.refusal(found) }
  end
  return { level = "ok", text = ("dam %s, inside >=%s <%s"):format(found, version.MIN, version.BELOW) }
end

---@param text string
---@param advice string|nil
---@return string
local function with_advice(text, advice)
  return advice and (text .. "; " .. advice) or text
end

---@param rows table[]
---@return string
local function rows_text(rows)
  return table.concat(vim.tbl_map(function(row)
    return "\n" .. with_advice(("  %s (%s): %s"):format(row.name, row.result, row.detail), row.advice)
  end, rows))
end

---@param doc damnit.Doctor
---@return damnit.HealthItem[]
function M.checks(doc)
  return vim.tbl_map(function(check)
    local level = LEVELS[check.result]
    local first = check.name .. ": " .. check.detail
    if level == "warn" or level == "error" then
      return { level = level, text = first .. rows_text(check.rows), advice = check.advice }
    end
    return { level = level, text = with_advice(first, check.advice) .. rows_text(check.rows) }
  end, doc.checks)
end

---@param want "auto"|"rex"|"clipboard"
---@param chosen string the name of the host a hand-off would use
---@param unavailable string|nil why the chosen host cannot take the brief
---@param rex_why string|nil why rex is not available
---@return damnit.HealthItem
function M.host(want, chosen, unavailable, rex_why)
  if unavailable then
    return {
      level = "warn",
      text = "hand-off falls back to the clipboard; " .. unavailable,
      advice = 'run Neovim in a Rex pane, or set handoff = { host = "clipboard" }',
    }
  end
  if want == "auto" and rex_why then
    return { level = "info", text = ("hand-off goes to the %s; %s"):format(chosen, rex_why) }
  end
  return { level = "ok", text = "hand-off goes to " .. (chosen == "clipboard" and "the clipboard" or chosen) }
end

return M
