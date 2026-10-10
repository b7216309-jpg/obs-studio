# Feature sources

Other projects we can borrow features, UX and code from. Before copying code,
check that its license is compatible with GPL-2.0-or-later and keep its
copyright notice; borrowing an idea or a design needs neither.

| Project | License | What to borrow |
|---|---|---|
| [omacom/monologue](https://github.com/omacom/monologue) | MIT | See below |

## omacom/monologue

A Qt 6 / C++17 webcam recorder for Omarchy (Linux). Its record-then-edit flow
is much simpler than OBS's for a single camera take. Features worth copying:

- **Take editor after recording:** stop a take and it opens in a built-in
  editor: split it into clips on a filmstrip, trim each clip with handles,
  remove clips, undo and redo. Edits never touch the original file.
- **Pause marks:** pausing and resuming a take is recorded on the timeline,
  and split points and trim handles snap to those marks.
- **Device memory by ID:** the camera and microphone are remembered by device
  ID, and a missing device is reported instead of silently switching to
  another one.
- **Automatic capture format:** picks the camera's highest advertised
  resolution, preferring 30 fps at that resolution.
- **Microphone meter:** live level with peak hold and clipping indication,
  still running while paused, without monitoring through the speakers.
- **Explicit "No audio" option** for silent recordings.
- **Atomic save** of the H.264/AAC MP4, keeping the original recording.
- **Keyboard-first controls:** Space records, pauses and resumes; Enter stops
  and opens the editor; editor keys for split, remove, jump to the previous or
  next pause, zoom and save.
