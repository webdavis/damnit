local highlights = require("damnit.kernel.highlights")

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

local function linked()
  local got = {}
  for group in pairs(LINKS) do
    got[group] = vim.api.nvim_get_hl(0, { name = group, link = true }).link
  end
  return got
end

return {
  ["setup links every Dam group to a standard group"] = function()
    require("damnit").setup({})
    assert(vim.deep_equal(linked(), LINKS), vim.inspect(linked()))
  end,

  ["a colorscheme change links them again"] = function()
    highlights.define()
    vim.cmd.highlight("clear")
    vim.cmd.doautocmd("ColorScheme")
    assert(vim.deep_equal(linked(), LINKS), vim.inspect(linked()))
  end,

  ["a group the person set keeps their colour"] = function()
    vim.api.nvim_set_hl(0, "DamDue", { fg = "#ff0000" })
    highlights.define()
    local due = vim.api.nvim_get_hl(0, { name = "DamDue", link = true })
    assert(due.link == nil and due.fg == 0xff0000, vim.inspect(due))
    vim.api.nvim_set_hl(0, "DamDue", { link = "Constant" })
  end,
}
