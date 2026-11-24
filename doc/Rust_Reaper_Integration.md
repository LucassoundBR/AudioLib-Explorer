# Send to REAPER on Windows

In AudioLib Explorer, use **Settings → Export REAPER extension…** to show
`reaper_audiolib_bridge.dll` in Explorer. Close REAPER, copy the DLL into
`UserPlugins` in its resource directory, then reopen REAPER. This is a one-time
setup. The extension uses REAPER's native plug-in API.

Keep one REAPER instance open, select exactly one destination track and place
the edit cursor. Select a local sound in AudioLib and press **S**. The classic
audition toolbar, including in Default Lucas, also has a **Send to REAPER** icon.
With a region selected, both controls use its prepared WAV. For a server sound,
select a region and let its WAV finish preparing.

Enable **Auto-Focus** next to the Send icon to bring REAPER forward after an
acknowledged insertion. It starts unchecked. If Windows denies the focus change,
the audio remains inserted. REAPER groups the insertion into a single Undo step
and preserves the edit cursor.

AudioLib retains exported slices after submission because REAPER may reference
them. REAPER's media-copy preferences determine whether the project has its own
copy. If AudioLib reports that insertion could not be confirmed, check the
project before retrying to avoid inserting another copy.
