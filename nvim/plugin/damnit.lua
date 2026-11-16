vim.keymap.set("n", "<Plug>(dam-today)", "<Cmd>Dam today<CR>")
vim.keymap.set("n", "<Plug>(dam-agenda)", "<Cmd>Dam agenda<CR>")
vim.keymap.set("n", "<Plug>(dam-plan)", "<Cmd>Dam plan<CR>")
vim.keymap.set("n", "<Plug>(dam-sidebar)", "<Cmd>Dam sidebar<CR>")
vim.keymap.set("n", "<Plug>(dam-capture)", "<Cmd>Dam capture<CR>")
vim.keymap.set("x", "<Plug>(dam-capture)", ":Dam capture<CR>", { silent = true })

if vim.fn.exists(":Dam") == 2 then
  return
end

vim.api.nvim_create_user_command("Dam", function(cmd)
  require("damnit").setup({})
  vim.cmd({ cmd = "Dam", args = cmd.fargs, range = cmd.range > 0 and { cmd.line1, cmd.line2 } or nil, mods = cmd.smods })
end, {
  nargs = "*",
  range = true,
  desc = "dam",
  complete = function(_, line)
    require("damnit").setup({})
    return vim.fn.getcompletion(line, "cmdline")
  end,
})
