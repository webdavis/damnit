local M = {}

local function padded(text)
  return text == "" and "" or (" %s "):format(text)
end

local function wrapped(text, width)
  local lines, line = {}, ""
  for word in text:gmatch("%S+") do
    if line ~= "" and vim.fn.strdisplaywidth(line .. " " .. word) > width then
      table.insert(lines, line)
      line = word
    else
      line = line == "" and word or (line .. " " .. word)
    end
  end
  table.insert(lines, line)
  return lines
end

local INPUT_HEIGHT = 3

local function at_cursor()
  local textoff = vim.fn.getwininfo(vim.api.nvim_get_current_win())[1].textoff
  local below = vim.api.nvim_win_get_height(0) - vim.fn.winline() >= INPUT_HEIGHT
  return {
    relative = "cursor",
    anchor = below and "NW" or "SW",
    row = below and 1 or 0,
    col = textoff - vim.fn.wincol() + 1,
    width = math.max(vim.api.nvim_win_get_width(0) - textoff - 2, 20),
  }
end

local function placed(anchor)
  if anchor == "cursor" then
    return at_cursor()
  end
  if anchor and vim.api.nvim_win_is_valid(anchor) then
    local width = math.max(vim.api.nvim_win_get_width(anchor) - 2, 20)
    return {
      relative = "win",
      win = anchor,
      anchor = "SW",
      row = vim.api.nvim_win_get_height(anchor) - 1,
      col = 0,
      width = width,
    }
  end
  local width = math.max(math.min(vim.o.columns - 4, 100), 20)
  local col = math.floor((vim.o.columns - width - 2) / 2)
  return { relative = "editor", row = math.floor((vim.o.lines - 3) / 4), col = col, width = width }
end

---@param buf integer
---@param title string
---@param anchor? integer|"cursor"
---@return integer win
function M.input(buf, title, anchor)
  return vim.api.nvim_open_win(
    buf,
    true,
    vim.tbl_extend("force", placed(anchor), {
      height = 1,
      style = "minimal",
      border = "rounded",
      title = padded(title),
      title_pos = "left",
      footer = "",
      footer_pos = "right",
    })
  )
end

---@param buf integer
---@param title string
---@param footer string
---@return integer win
function M.sheet(buf, title, footer)
  local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  local widest = math.max(vim.fn.strdisplaywidth(title), vim.fn.strdisplaywidth(footer)) + 4
  for _, line in ipairs(lines) do
    widest = math.max(widest, vim.fn.strdisplaywidth(line) + 2)
  end
  local width = math.min(widest, vim.o.columns - 4)
  local height = math.min(#lines, vim.o.lines - 6)
  return vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    row = math.floor((vim.o.lines - height - 2) / 2),
    col = math.floor((vim.o.columns - width - 2) / 2),
    width = width,
    height = height,
    style = "minimal",
    border = "rounded",
    title = padded(title),
    title_pos = "left",
    footer = padded(footer),
    footer_pos = "right",
  })
end

---@param win integer
---@param text string
---@param group string
---@param more? string[] lines shown under the input
function M.footer(win, text, group, more)
  if not vim.api.nvim_win_is_valid(win) then
    return
  end
  local buf, width = vim.api.nvim_win_get_buf(win), vim.api.nvim_win_get_width(win)
  local namespace = vim.api.nvim_create_namespace("damnit.prompt")
  vim.api.nvim_buf_clear_namespace(buf, namespace, 0, -1)
  local long = vim.fn.strdisplaywidth(text) > width - 4
  local below = vim.tbl_map(function(line)
    return { { line, group } }
  end, long and wrapped(text, width) or {})
  for _, line in ipairs(more or {}) do
    table.insert(below, { { line } })
  end
  if #below > 0 then
    vim.api.nvim_buf_set_extmark(buf, namespace, 0, 0, { virt_lines = below })
  end
  local footer = (text == "" or long) and "" or { { padded(text), group } }
  vim.api.nvim_win_set_config(win, { footer = footer, footer_pos = "right", height = 1 + #below })
end

return M
