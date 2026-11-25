# Waveform fade handles

The lower Waveform can support non-destructive fades on an entire local sound
or a selected range. The two values start at zero for each new edit target.
The left handle controls fade-in in playback order and the right handle
controls fade-out. Reverse keeps each handle on its visual side. Dragging a
handle changes its duration in source frames and stops before it overlaps the
other fade.

## Interaction and preview

Use the top corners of the visible edit target in
`app/audiolib-desktop/ui/audition-panel.slint`. Give each handle a small, separate
hit region above the selection edge and the body that owns drag-to-export.
Capture the target and viewport on press. Update the drawn curve during drag
and commit the result on release. Escape cancels the draft. Show each handle
when its boundary is visible. Keep its hit region disjoint from resize and
export gestures at 0 ms and at minimum selection width. The first version uses
linear gain. This keeps the displayed curve, audition and exported samples
tied to the same formula.

Use the original, half-open source frame range for fade lengths. The first
faded frame is silent at the leading edge, and the last faded frame is silent
at the trailing edge. A fade of zero keeps that edge at unity gain. In
Reverse, the elapsed frame at source frame `i` is `end - 1 - i`. A seek must
recompute elapsed time against the edit target's original bounds. Using the
decoder's remaining range after seek would restart the fade incorrectly.

`prepare_audition_decoder` in `crates/audiolib-audio/src/engine.rs` is where the
active range and seek offset are available together. Keep both the original
mapped range and the remaining decoder range when passing the fade to
`AuditionDecoder::routed`. Apply the shared envelope to the actual decoded
frames returned by `ResampledDecoder::read_input` in
`crates/audiolib-audio/src/decoder.rs`. This is after ordering the frames and
before padding, resampling and SoundTouch. Advance the envelope by source
frames read, including when the resampler needs another block before emitting
output. The realtime callback continues to consume prepared samples. A change
to an active fade should retain the play session and use the existing source
transition to refresh buffered samples at the current position.

## Slice and REAPER output

Keep edit values with the source/range identity in the desktop controller.
When they change, cancel the obsolete slice render and invalidate its
published cache using the same pattern as Reverse in
`app/audiolib-desktop/src/waveform_selection.rs`. Both drag-to-export and the
REAPER button and shortcut already consume the prepared WAV. Use that same
file for faded slices. A full-file send with an active fade also needs a
rendered WAV. Generalize the whole-file Reverse staging path for that case and
carry the fade revision through send and publication checks. Reject a send
while a newer envelope is pending, so a shortcut immediately after dragging
cannot send the previous render.

Apply fades in export order in `crates/audiolib-audio/src/slice.rs`. Keep the
zero-fade copy path exact. Scale integer PCM in its significant bits, retain
left-aligned padding and apply one gain per frame to every channel. The
native float path can retain 64-bit precision for 64-bit source PCM. Apply
the same envelope to the forward decoded FLAC and float paths. Reverse
compressed audio already spools a forward WAV, where the envelope can be
applied in the final directional pass. Keep the existing format, file-lease,
staging and cancellation rules.

Start with local audio. Server-stream audition and direct PCM range copies
need explicit coverage before enabling the handles there.

## Evidence and next checks

A standalone PCM experiment passed six tests for asymmetric fades in both
directions, seek offset, zero and one-frame fades, 24- and 32-bit signed
rounding, and silence in unsigned PCM. It also preserved padding for 24
valid bits stored in a 32-bit word. In one test, an in-memory render of a
192,000-frame reversed region from the dedicated 48 kHz, 24-bit stereo WAV
retained the header format, channel order and exact unity samples. This
experiment ran outside the application's UI, renderer and audio engine.

The implementation compares decoded audition against a re-opened 24-bit stereo
WAV for a nonzero selection in both directions, including a seek inside the
leading fade. Interior samples match and ramp samples stay within half a
24-bit quantization step, plus float tolerance. The audio package suite
passed 211 tests.

The isolated Slint test drags handles for the whole file and a reversed
selection in both player layouts. It verifies that Escape cancels the draft
and that editing a fade does not commit another selection. The desktop
controller test also verifies that changing the fade retires the prepared
Reverse export. It re-opens the WAV used for drag and REAPER send and checks
the endpoints, ramp samples and untouched interior of the mono PCM output.
Both tests passed in the ASIO desktop configuration without a physical stream.

Live playback and final delivery to REAPER are pending interactive
acceptance. The first implementation exposes the handles for local sounds.
