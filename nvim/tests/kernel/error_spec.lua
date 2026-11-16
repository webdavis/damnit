local errors = require("damnit.kernel.error")
local fixtures = require("helpers.fixtures")

return {
  ["an error document after progress records is read as that error"] = function()
    local err = errors.of_exit(1, fixtures.read("progress-push-failed.txt"))
    assert(err.kind == "helper", vim.inspect(err))
    assert(err.message == "the remote refused: fake is rate limiting this account; retry after 42s", err.message)
  end,

  ["text after progress records is kept as the plugin error without the records"] = function()
    local records = fixtures.read("progress-push.txt")
    local err = errors.of_exit(1, records .. "dam: killed\n")
    assert(err.kind == "plugin" and err.message == "dam: killed", vim.inspect(err))
  end,
}
