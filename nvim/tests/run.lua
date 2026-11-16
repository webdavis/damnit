local tests_dir = vim.fs.dirname(vim.fs.abspath(arg[0]))
local plugin_dir = vim.fs.dirname(tests_dir)
local only = arg[1]

package.path = ("%s/lua/?.lua;%s/lua/?/init.lua;%s/?.lua;%s"):format(plugin_dir, plugin_dir, tests_dir, package.path)

local function spec_files()
  if only then
    return { vim.fs.abspath(only) }
  end
  local files = vim.fn.globpath(tests_dir, "**/*_spec.lua", false, true)
  table.sort(files)
  return files
end

local function cases_of(path)
  local cases = dofile(path)
  if type(cases) ~= "table" or next(cases) == nil then
    error(path .. " returned no cases")
  end
  local names = vim.tbl_keys(cases)
  table.sort(names)
  return cases, names
end

vim.o.messagesopt = "wait:0,history:500"

local notify = vim.notify

local function closed_floats()
  for _, win in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(win).relative ~= "" then
      vim.api.nvim_win_close(win, true)
    end
  end
end

local function fresh_editor()
  vim.notify = notify
  closed_floats()
  vim.cmd("silent! %bwipeout!")
  vim.api.nvim_create_augroup("DamSpec", { clear = true })
end

local files = spec_files()
if #files == 0 then
  error("no spec files under " .. tests_dir)
end

local passed, failed, skipped = 0, 0, 0
for _, path in ipairs(files) do
  local spec = path:sub(#tests_dir + 2, -5)
  local cases, names = cases_of(path)
  for _, name in ipairs(names) do
    fresh_editor()
    local ok, result = xpcall(cases[name], debug.traceback)
    if not ok then
      failed = failed + 1
      io.write(("FAIL %s: %s: %s\n"):format(spec, name, result))
    elseif type(result) == "string" then
      skipped = skipped + 1
      io.write(("skip %s: %s: %s\n"):format(spec, name, result))
    else
      passed = passed + 1
      io.write(("ok %s: %s\n"):format(spec, name))
    end
  end
end

io.write(("%d passed, %d failed, %d skipped\n"):format(passed, failed, skipped))
os.exit(failed == 0 and 0 or 1)
