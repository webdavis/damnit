local fixtures = require("helpers.fixtures")
local winbar = require("damnit.status.winbar")

local function text_of(chunks)
  return table.concat(vim.tbl_map(function(chunk)
    return chunk[1]
  end, chunks))
end

return {
  ["the status winbar counts each section that holds something"] = function()
    local text = text_of(winbar.chunks(fixtures.decoded("status.json")))
    assert(text == " status · 1 conflict · 2 waiting · 1 unstaged · 1 staged · 2 unpushed commits", text)
  end,

  ["an empty status says it is clean"] = function()
    local text = text_of(winbar.chunks(fixtures.decoded("status-empty.json")))
    assert(text == " status · clean", text)
  end,

  ["conflicts take the overdue colour and the name the header colour"] = function()
    local chunks = winbar.chunks(fixtures.decoded("status.json"))
    assert(vim.deep_equal(chunks[1], { " status", "DamHeader" }), vim.inspect(chunks))
    assert(vim.deep_equal(chunks[4], { "1 conflict", "DamOverdue" }), vim.inspect(chunks))
  end,

  ["the status winbar is cut after its name"] = function()
    local chunks = winbar.chunks(fixtures.decoded("status.json"))
    assert(vim.deep_equal(chunks[2], { "", cut = true }), vim.inspect(chunks))
  end,
}
