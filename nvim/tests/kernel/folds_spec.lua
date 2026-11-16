local folds = require("damnit.kernel.folds")

return {
  ["a header opens a fold that holds its rows"] = function()
    local levels = folds.levels({ { header = true }, { depth = 0 }, { depth = 0 }, { header = true }, { depth = 0 } })
    assert(vim.deep_equal(levels, { ">1", 1, 1, ">1", 1 }), vim.inspect(levels))
  end,

  ["a parent opens a fold one deeper that holds its children"] = function()
    local levels = folds.levels({ { header = true }, { depth = 0 }, { depth = 1 }, { depth = 1 }, { depth = 0 } })
    assert(vim.deep_equal(levels, { ">1", ">2", 2, 2, 1 }), vim.inspect(levels))
  end,

  ["without headers only parents fold"] = function()
    local levels = folds.levels({ { depth = 0 }, { depth = 1 }, { depth = 2 }, { depth = 0 } })
    assert(vim.deep_equal(levels, { ">1", ">2", 2, 0 }), vim.inspect(levels))
  end,

  ["a blank line between sections closes the fold above it"] = function()
    local levels = folds.levels({ { header = true }, { depth = 0 }, {}, { header = true }, { depth = 0 } })
    assert(vim.deep_equal(levels, { ">1", 1, 0, ">1", 1 }), vim.inspect(levels))
  end,
}
