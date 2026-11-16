local M = {}

local LINKS = {
  DamOverdue = "DiagnosticError",
  DamBlocked = "DiagnosticWarn",
  DamDone = "Comment",
  DamHeader = "Title",
  DamPriority = "Special",
  DamLabel = "Identifier",
  DamCategory = "Type",
  DamDue = "Constant",
  DamMeta = "Comment",
  DamMarked = "Visual",
  DamAdded = "Added",
  DamChanged = "Changed",
  DamRemoved = "Removed",
}

function M.define()
  for group, link in pairs(LINKS) do
    vim.api.nvim_set_hl(0, group, { link = link, default = true })
  end
end

function M.setup()
  M.define()
  vim.api.nvim_create_autocmd("ColorScheme", {
    group = vim.api.nvim_create_augroup("damnit.highlights", { clear = true }),
    callback = M.define,
  })
end

return M
