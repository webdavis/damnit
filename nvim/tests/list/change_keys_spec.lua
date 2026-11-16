local buffer = require("helpers.buffer")
local list = require("helpers.list")

local MOM, MILK, BAG = list.CALL_MOM, list.OAT_MILK, list.BAG

local function same(got, want)
  assert(vim.deep_equal(got, want), vim.inspect(got))
end

return {
  ["x on one row sends dam done with its oid at once"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("x")
    same(list.wait_sent(fake, "done", 1), { { argv = { "done", MOM, "-o", "json" } } })
    assert(#list.sent(fake, "done", true) == 0, "it previewed one row")
  end,

  ["Vjx previews the two rows, then y sends one dam done with both oids on stdin"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("Vjx")
    list.wait_confirm(fake, "done")
    same(list.sent(fake, "done", true)[1].argv, { "done", "-", "--expect", "2", "--preview", "-o", "json" })
    buffer.feed("y")
    local writes = list.wait_sent(fake, "done", 1)
    same(writes, { { argv = { "done", "-", "--expect", "2", "-o", "json" }, stdin = MOM .. "\n" .. MILK .. "\n" } })
  end,

  ["3x is one call for three rows"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("3x")
    list.wait_confirm(fake, "done")
    buffer.feed("y")
    local writes = list.wait_sent(fake, "done", 1)
    assert(#writes == 1 and writes[1].stdin == table.concat({ MOM, MILK, BAG }, "\n") .. "\n", vim.inspect(writes))
  end,

  ["x. sends dam done twice"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("x")
    list.wait_sent(fake, "done", 1)
    buffer.feed(".")
    same(list.wait_sent(fake, "done", 2), {
      { argv = { "done", MOM, "-o", "json" } },
      { argv = { "done", MOM, "-o", "json" } },
    })
  end,

  ["X reopens, p2 sets the priority and s stages the row"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("X")
    same(list.wait_sent(fake, "edit", 1)[1].argv, { "edit", MOM, "--no-done", "-o", "json" })
    buffer.feed("p2")
    same(list.wait_sent(fake, "edit", 2)[2].argv, { "edit", MOM, "--priority", "2", "-o", "json" })
    buffer.feed("s")
    same(list.wait_sent(fake, "add", 1)[1].argv, { "add", MOM, "-o", "json" })
  end,

  ["p2 on three marked rows changes all three in one call"] = function()
    local _, fake = list.opened()
    list.on("[ ] call mom")
    buffer.feed("3m")
    list.on("[ ] p2 buy oat milk")
    buffer.feed("p2")
    list.wait_confirm(fake, "edit")
    buffer.feed("y")
    local writes = list.wait_sent(fake, "edit", 1)
    same(writes[1].argv, { "edit", "-", "--priority", "2", "--expect", "3", "-o", "json" })
  end,

  ["under a token a many-row call adds --as-one"] = function()
    local _, fake = list.opened()
    vim.env.DAM_TOKEN = "held"
    local ok, err = pcall(function()
      list.on("[ ] call mom")
      buffer.feed("2x")
      list.wait_confirm(fake, "done")
      same(
        list.sent(fake, "done", true)[1].argv,
        { "done", "-", "--expect", "2", "--preview", "--as-one", "-o", "json" }
      )
    end)
    vim.env.DAM_TOKEN = nil
    assert(ok, err)
  end,

  ["after a write the echo counts the rows and the cursor follows the task to its new line"] = function()
    local _, fake = list.opened()
    local notes = buffer.capture_notes()
    list.on("[ ] p2 buy oat milk")
    fake.reply("list", require("helpers.fixtures").answer("list-tree.json"))
    buffer.feed("x")
    buffer.wait_for(function()
      return #notes > 0 and buffer.lines()[1] ~= "[ ] call mom" or #buffer.lines() > 3
    end)
    vim.wait(30)
    same(notes[1].text, "dam: 1 done")
    same(vim.api.nvim_win_get_cursor(0)[1], 4)
    same(buffer.lines()[4], "[ ] p2 buy oat milk")
  end,

  ["when the task left the view the cursor keeps its line, or the last one"] = function()
    local _, fake = list.opened()
    list.on("  [ ] pack the bag")
    fake.reply("list", require("helpers.fixtures").answer("list-filter-today.json"))
    buffer.feed("x")
    buffer.wait_for(function()
      return #buffer.lines() == 1
    end)
    vim.wait(30)
    same(vim.api.nvim_win_get_cursor(0)[1], 1)
  end,
}
