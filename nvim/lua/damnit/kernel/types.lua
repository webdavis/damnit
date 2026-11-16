---@class damnit.Options
---@field dam string
---@field views table<string, string|damnit.ViewSpec>
---@field default_view? string|damnit.ViewSpec
---@field timeout_ms integer
---@field sidebar { side: "left"|"right", width: integer }
---@field handoff { host: "auto"|"rex"|"clipboard" }
---@field signs { overdue: string, blocked: string, changed: string, staged: string, waiting: string }

---@class damnit.App
---@field options damnit.Options
---@field dam damnit.Dam
---@field queue damnit.Queue
---@field screen damnit.Screen
---@field keys damnit.Keys
---@field say damnit.Say
---@field vocab damnit.VocabCache

---@class damnit.Feature
---@field commands table<string, fun(args: string[], cmd: table)>
---@field complete? table<string, fun(arglead: string): string[]>
---@field default? fun(args: string[], cmd: table)

---@class damnit.Call
---@field args string[]
---@field cwd string
---@field placed? boolean the caller shows a refusal where it belongs, so the write only keeps it
---@field stdin? string
---@field key? string
---@field progress? boolean

---@class damnit.Handle
---@field cancel fun()

---@class damnit.Dam
---@field run fun(self, call: damnit.Call, done: fun(answer: table|nil, err: damnit.Error|nil)): damnit.Handle
---@field write fun(self, call: damnit.Call, done: fun(answer: table|nil, err: damnit.Error|nil)): damnit.Handle
---@field version fun(self): string|nil

---@class damnit.Queue
---@field submit fun(self, lane: string, start: fun(finish: fun()): damnit.Handle, mode: "latest"|"ordered")
---@field cancel fun(self, lane?: string): integer

---@class damnit.Screen
---@field scratch fun(self, name: string, filetype: string): integer
---@field draw fun(self, buf: integer, lines: string[], marks: damnit.Mark[])
---@field show fun(self, buf: integer, smods?: table)
---@field window fun(self, buf: integer, options: table<string, string|boolean>)
---@field text_width fun(self, buf: integer): integer
---@field bar fun(chunks: string[][]): string

---@class damnit.Mark
---@field line integer
---@field col? integer
---@field end_col? integer
---@field group? string
---@field virt? string[][]
---@field align? "eol"|"eol_right_align"|"right_align"|"overlay"|"inline"
---@field sign? string
---@field sign_group? string
---@field line_group? string
---@field virt_lines? string[][][]

---@class damnit.KeySpec
---@field lhs string
---@field desc string
---@field run fun()
---@field mode? string|string[]
---@field group? string the heading g? lists it under

---@class damnit.Keys
---@field bind fun(self, buf: integer, specs: damnit.KeySpec[])

---@class damnit.Say
---@field info fun(self, text: string)
---@field warn fun(self, text: string)
---@field error fun(self, err: damnit.Error)
---@field keep fun(self, err: damnit.Error) logs it for :Dam messages without announcing it
---@field log fun(self): damnit.Error[]
---@field changed fun(self, cwd: string)

---@class damnit.Selection
---@field field string
---@field value string
---@field shapes string[]
---@field command string

---@class damnit.Error
---@field kind string
---@field rule? string
---@field message string
---@field oids string[]
---@field usage? { problem: string, word: string|nil }
---@field selection? damnit.Selection
---@field prefix? string
---@field key? string
---@field hook? string

---@class damnit.ViewSpec
---@field global? boolean

---@class damnit.Object
---@field oid string
---@field kind "task"|"event"
---@field subject string
---@field path string
---@field labels string[]
---@field parent? string
---@field category table<string, string>
---@field overdue boolean
---@field blocked boolean
---@field task? { done: boolean, priority: integer, due?: string, deadline?: string, estimate?: string }
---@field event? { start: string }

---@class damnit.ListAnswer
---@field objects damnit.Object[]

---@class damnit.StatusChange
---@field oid string
---@field op "create"|"update"|"delete"
---@field fields string[]
---@field kind "task"|"event"
---@field subject string
---@field path string

---@class damnit.StatusWaiting: damnit.StatusChange
---@field hash string
---@field item "new"|"change"|"removal"
---@field from string
---@field age integer

---@class damnit.StatusSet
---@field set string
---@field from string
---@field count integer
---@field age integer

---@class damnit.StatusDoc
---@field conflicts { oid: string, remote: string, ours?: damnit.Object, theirs?: damnit.Object }[]
---@field staged damnit.StatusChange[]
---@field unstaged damnit.StatusChange[]
---@field waiting damnit.StatusWaiting[]
---@field sets damnit.StatusSet[]
---@field unpushed { remote: string, commits: integer }[]
---@field notices { kind: string, remote?: string }[]
---@field warnings string[]

---@class damnit.VocabCache
---@field words fun(self, cwd: string): damnit.Vocab
---@field flags fun(self, command: string): damnit.Flag[]
