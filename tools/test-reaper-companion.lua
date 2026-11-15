-- tools/test-reaper-companion.lua
-- Run with Lua 5.4. Executes the actual companion against an in-memory host;
-- never starts REAPER, edits a real project, opens a device or activates a window.
local file = assert(io.open(arg[1] or 'app/audiolib-desktop/reaper/AudioLib_Explorer_Bridge.lua', 'rb'))
local source = file:read('*a'); file:close()
local function hex(s) return (s:gsub('.', function(c) return string.format('%02x', c:byte()) end)) end
source = source:gsub('__BRIDGE_ROOT_HEX__', hex('owned-test'))
local header, session = 'ALE_REAPER_1', string.rep('a', 32)
local function host(options)
  options = options or {}
  local h = { files = {}, inserts = {}, messages = {}, cursor = 2.5, undo = 0, now = 1000 }
  local project = {}
  local root = 'owned-test/'
  local env = setmetatable({}, { __index = _G })
  env.os = {
    time = function() return h.now end,
    remove = function(path) h.files[path] = nil; return true end,
    rename = function(from, to)
      if h.before_claim and from == root .. 'request.txt' then
        h.before_claim(); h.before_claim = nil
      end
      if not h.files[from] or h.files[to] then return nil, 'unavailable' end
      h.files[to] = h.files[from]; h.files[from] = nil; return true
    end,
  }
  env.io = { open = function(path, mode)
    if mode == 'rb' then
      if not h.files[path] then return nil end
      local bytes = h.files[path]
      return { read = function(_, n) return bytes:sub(1, n) end, close = function() end }
    end
    assert(mode == 'wb'); h.files[path] = ''
    return { write = function(_, text) h.files[path] = h.files[path] .. text end, close = function() end }
  end }
  env.reaper = {
    genGuid = function() return session end,
    GetMainHwnd = function() return 'userdata: 0x123' end,
    time_precise = function() return h.now end,
    atexit = function(fn) h.exit = fn end,
    defer = function(fn) h.tick = fn end,
    ShowMessageBox = function(text) h.messages[#h.messages + 1] = text end,
    ShowConsoleMsg = function(text) error(text) end,
    file_exists = function() return not options.missing end,
    EnumProjects = function() return project, options.unsaved and '' or 'Project 音效.rpp' end,
    GetSelectedTrack = function(p) assert(p == project); return not options.no_track and {} or nil end,
    GetCursorPositionEx = function(p) assert(p == project); return h.cursor end,
    Undo_BeginBlock2 = function(p) assert(p == project); h.undo = h.undo + 1 end,
    Undo_EndBlock2 = function(p) assert(p == project); h.undo = h.undo - 1 end,
    InsertMedia = function(path, mode) assert(mode == 0); h.inserts[#h.inserts + 1] = path; return 1 end,
    SetEditCurPos2 = function(p, cursor, move, seek) assert(p == project and not move and not seek); h.cursor = cursor end,
    UpdateArrange = function() end,
  }
  if options.existing then h.files[root .. 'session.txt'] = header .. '\n' .. session .. '\nwindow\n1000\n' end
  assert(load(source, '@AudioLib_Explorer_Bridge.lua', 't', env))()
  function h.request(id, path, expiry, target)
    h.files[root .. 'request.txt'] = table.concat({header, target or session, id, expiry or 1012, hex(path)}, '\n') .. '\n'
  end
  function h.reply(id) return h.files[root .. id .. '.reply'] or '' end
  return h
end
local passed = 0
local function test(name, fn) fn(); passed = passed + 1; print('PASS ' .. name) end
local id, other = string.rep('b', 32), string.rep('c', 32)
test('original Unicode path, one import, cursor and undo preserved', function()
  local h = host(); local path = "C:/Sounds/seleção '音效'.wav"
  h.request(id, path); h.tick(); h.tick()
  assert(#h.inserts == 1 and h.inserts[1] == path and h.cursor == 2.5 and h.undo == 0)
  assert(h.reply(id):find('\nOK\n', 1, true)); assert(not h.files['owned-test/request.txt'])
  h.exit(); assert(not h.files['owned-test/session.txt'])
end)
test('no destination track reports error without insertion', function()
  local h = host({no_track = true}); h.request(id, 'source.wav'); h.tick()
  assert(#h.inserts == 0 and h.undo == 0 and h.reply(id):find('\nERROR\n', 1, true))
end)
test('expired request is rejected', function()
  local h = host(); h.request(id, 'source.wav', 999); h.tick()
  assert(#h.inserts == 0 and h.reply(id):find('\nERROR\n', 1, true))
end)
test('missing original is rejected', function()
  local h = host({missing = true}); h.request(id, 'missing.wav'); h.tick()
  assert(#h.inserts == 0 and h.reply(id):find('\nERROR\n', 1, true))
end)
test('foreign session cannot import', function()
  local h = host(); h.request(id, 'source.wav', nil, other); h.tick()
  assert(#h.inserts == 0 and h.files['owned-test/request.txt'])
end)
test('duplicate companion refuses a fresh active session', function()
  local h = host({existing = true}); assert(not h.tick and #h.messages == 1)
end)
test('claim race uses actual claimed request and acknowledges its ID', function()
  local h = host(); h.request(id, 'cancelled.wav')
  h.before_claim = function() h.request(other, 'replacement.wav') end
  h.tick(); assert(#h.inserts == 1 and h.inserts[1] == 'replacement.wav')
  assert(h.reply(other):find('\nOK\n', 1, true) and h.reply(id) == '')
end)
test('unsaved project and Windows verbatim path', function()
  local h = host({unsaved = true}); h.request(id, '\\\\?\\C:\\Sounds\\source.wav'); h.tick()
  assert(h.inserts[1] == 'C:\\Sounds\\source.wav' and h.reply(id):find('\nOK\n\n', 1, true))
end)
print(string.format('%d companion checks passed; native REAPER not exercised.', passed))
