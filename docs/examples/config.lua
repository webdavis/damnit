return {
  owner = "Ada",
  done = { interactive = false },
  remote = {
    todoist = { url = "todoist::", deadline = "60s", path = "work" },
  },
  category = {
    effort = { values = { "light", "admin", "deep" }, exclusive = true },
  },
  filter = {
    today = { query = "due:today | overdue" },
  },
  -- hook = {
  --   task_done = function(task) print("done: " .. task.subject) end,
  -- },
}
