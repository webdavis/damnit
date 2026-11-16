local align = require("damnit.kernel.align")

local CHUNKS = { { "  " }, { "2026-09-25", "DamDue" } }

return {
  ["the columns sit at the right edge over the row"] = function()
    local mark = align.right(3, 10, CHUNKS, 40)
    assert(vim.deep_equal(mark, { line = 3, virt = CHUNKS, align = "right_align" }), vim.inspect(mark))
  end,

  ["a row too long for its columns ends in an ellipsis before them"] = function()
    local mark = align.right(0, 30, CHUNKS, 40)
    assert(vim.deep_equal(mark.virt, { { "… ", "DamMeta" }, { "2026-09-25", "DamDue" } }), vim.inspect(mark))
    assert(vim.deep_equal(CHUNKS[1], { "  " }), "the chunks handed in were changed")
  end,

  ["a row exactly as wide as the room keeps its gap"] = function()
    local mark = align.right(0, 28, CHUNKS, 40)
    assert(vim.deep_equal(mark.virt, CHUNKS), vim.inspect(mark))
  end,

  ["no columns on a row that fits gives no mark"] = function()
    assert(align.right(0, 10, {}, 40) == nil)
    assert(align.right(0, 10, { { "  " }, { "     " } }, 40) == nil)
  end,

  ["the mark begins at the row's first value and keeps the padding after it"] = function()
    local chunks = { { "  " }, { "      " }, { "  " }, { "45m", "DamMeta" }, { "  " }, { "          " } }
    local mark = align.right(0, 10, chunks, 40)
    assert(vim.deep_equal(mark.virt, { { "  " }, { "45m", "DamMeta" }, { "  " }, { "          " } }), vim.inspect(mark))
  end,

  ["a row wider than the window with nothing on the right ends in an ellipsis at the edge"] = function()
    local mark = align.right(2, 50, { { "  " }, { "          " } }, 40)
    assert(
      vim.deep_equal(mark, { line = 2, virt = { { "…", "DamMeta" } }, align = "right_align" }),
      vim.inspect(mark)
    )
  end,
}
