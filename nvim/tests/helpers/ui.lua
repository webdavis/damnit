local M = {}

---@param answers { input?: string, select?: string }
---@param run fun(prompts: string[])
function M.answering(answers, run)
  local input, select = vim.ui.input, vim.ui.select
  local prompts = {}
  vim.ui.input = function(opts, on_confirm)
    table.insert(prompts, opts.prompt)
    on_confirm(answers.input)
  end
  vim.ui.select = function(_, opts, on_choice)
    table.insert(prompts, opts.prompt)
    on_choice(answers.select)
  end
  local ok, err = xpcall(run, debug.traceback, prompts)
  vim.ui.input, vim.ui.select = input, select
  assert(ok, err)
end

return M
