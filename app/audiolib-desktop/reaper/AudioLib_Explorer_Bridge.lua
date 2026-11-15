-- AudioLib Explorer companion. Load and run through REAPER Actions once per session.
-- Local-only protocol; no playback, plugin loading, shell commands or network listener.
local function unhex(s)
  assert(#s % 2 == 0 and not s:find('[^%x]'), 'Invalid path encoding')
  return (s:gsub('..', function(c) return string.char(tonumber(c, 16)) end))
end
local function hex(s) return (s:gsub('.', function(c) return string.format('%02x', c:byte()) end)) end
local root = unhex('__BRIDGE_ROOT_HEX__')
local header = 'ALE_REAPER_1'
local session = reaper.genGuid():gsub('[^%x]', ''):lower()
local session_file = root .. '/session.txt'
local request_file = root .. '/request.txt'
local function read(path)
  local file = io.open(path, 'rb'); if not file then return nil end
  local text = file:read(16385); file:close()
  if text and #text <= 16384 then return text end
end
local function lines(text)
  local values = {}; for line in (text or ''):gmatch('([^\n]+)\n') do values[#values + 1] = line end
  return values
end
local function write(path, text)
  local file = assert(io.open(path, 'wb')); file:write(text); file:close()
end
local previous = lines(read(session_file))
local previous_age = tonumber(previous[4]) and os.time() - tonumber(previous[4])
if previous[1] == header and previous_age and previous_age >= 0 and previous_age <= 3 then
  reaper.ShowMessageBox('An AudioLib bridge is already connected. Stop that script before starting another instance.', 'AudioLib Explorer', 0)
  return
end
local function heartbeat()
  write(session_file, header .. '\n' .. session .. '\n' .. tostring(reaper.GetMainHwnd()) .. '\n' .. os.time() .. '\n')
end
heartbeat()
reaper.atexit(function()
  local current = lines(read(session_file))
  if current[2] == session then os.remove(session_file) end
end)
local last_heartbeat = 0
local function tick()
  local ok, error_text = xpcall(function()
    if reaper.time_precise() - last_heartbeat >= 0.25 then heartbeat(); last_heartbeat = reaper.time_precise() end
    local text = read(request_file); if not text then return end
    local request = lines(text)
    if request[1] ~= header or #request ~= 5 or not request[3]:match('^%x+$') or #request[3] ~= 32 then return end
    if request[2] ~= session then return end
    local id = request[3]
    local claim = root .. '/' .. id .. '.claimed'
    if not os.rename(request_file, claim) then return end -- atomic claim before any project edit
    -- The client can cancel an unread request and enqueue another between our
    -- read and rename. Always process the bytes actually claimed, never a stale
    -- copy from the earlier read.
    request = lines(read(claim))
    if request[1] ~= header or #request ~= 5 or request[2] ~= session
      or not request[3]:match('^%x+$') or #request[3] ~= 32 then
      os.remove(claim); return
    end
    id = request[3]
    local undo_project = nil
    local success, result = xpcall(function()
      assert(tonumber(request[4]) and os.time() <= tonumber(request[4]), 'AudioLib send expired; nothing imported')
      local source = unhex(request[5])
      if source:sub(1, 8) == '\\\\?\\UNC\\' then source = '\\\\' .. source:sub(9)
      elseif source:sub(1, 4) == '\\\\?\\' then source = source:sub(5) end
      assert(not source:find('%z'), 'Invalid audio path')
      assert(reaper.file_exists(source), 'The original audio file is unavailable')
      local project, name = reaper.EnumProjects(-1, '')
      assert(project and reaper.GetSelectedTrack(project, 0), 'Select a destination track in REAPER first')
      local cursor = reaper.GetCursorPositionEx(project)
      reaper.Undo_BeginBlock2(project); undo_project = project
      assert(reaper.InsertMedia(source, 0) > 0, 'REAPER could not insert the audio file')
      reaper.SetEditCurPos2(project, cursor, false, false)
      reaper.Undo_EndBlock2(project, 'AudioLib: send audio', -1); undo_project = nil
      reaper.UpdateArrange()
      return header .. '\n' .. id .. '\nOK\n' .. hex(name or '') .. '\n' .. string.format('%.9f', cursor) .. '\n' .. tostring(reaper.GetMainHwnd()) .. '\n'
    end, debug.traceback)
    if undo_project then reaper.Undo_EndBlock2(undo_project, 'AudioLib: send audio failed', -1) end
    local reply = root .. '/' .. id .. '.reply'
    local pending = reply .. '.pending'
    write(pending, success and result or (header .. '\n' .. id .. '\nERROR\n' .. hex(result) .. '\n'))
    assert(os.rename(pending, reply))
    os.remove(claim)
  end, debug.traceback)
  if not ok then reaper.ShowConsoleMsg('AudioLib bridge stopped: ' .. error_text .. '\n'); return end
  reaper.defer(tick)
end
reaper.defer(tick)
