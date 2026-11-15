return {
  owner = "Ada",
  done = { interactive = false },
  remote = {
    todoist = { url = "todoist::", deadline = "60s", sync_every = "15m", path = "work" },
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
