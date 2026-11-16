local M = {}

local function keys(lhs)
  vim.api.nvim_feedkeys(vim.keycode(lhs), "n", false)
end

local function pum_or(lhs, run)
  return function()
    if vim.fn.pumvisible() == 1 then
      return keys(lhs)
    end
    run()
  end
end

---@param session damnit.PromptSession
local function insert_keys(session)
  local function close()
    session:close()
  end
  return {
    ["<CR>"] = function()
      if vim.fn.pumvisible() == 1 and vim.fn.complete_info({ "selected" }).selected >= 0 then
        return keys("<C-y>")
      end
      session:submit(false)
    end,
    ["<C-j>"] = function()
      session:submit(true)
    end,
    ["<Esc>"] = close,
    ["<C-c>"] = close,
    ["<Up>"] = pum_or("<C-p>", function()
      session:older()
    end),
    ["<Down>"] = pum_or("<C-n>", function()
      session:newer()
    end),
    ["<Tab>"] = pum_or("<C-n>", function()
      keys("<C-x><C-u>")
    end),
    ["<S-Tab>"] = pum_or("<C-p>", function() end),
  }
end

---@param session damnit.PromptSession
function M.bind(session)
  local opts = { buffer = session.buf, nowait = true }
  for lhs, run in pairs(insert_keys(session)) do
    vim.keymap.set("i", lhs, run, opts)
  end
  for _, lhs in ipairs({ "<Esc>", "q", "<C-c>" }) do
    vim.keymap.set("n", lhs, function()
      session:close()
    end, opts)
  end
  vim.keymap.set("n", "<CR>", function()
    session:submit(false)
  end, opts)
  vim.api.nvim_create_autocmd("WinLeave", {
    buffer = session.buf,
    once = true,
    callback = function()
      session:close()
    end,
  })
end

return M
