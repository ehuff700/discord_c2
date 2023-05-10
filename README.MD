# Known Bugs
-[ ] exit command doesn't properly exit the command sessions for unknown reasons, likely because the `SHELL_TYPE` static reference is not properly nullified on exit?

-[ ] stderr under `run_command` method in `process_handler.rs` is not properly displayed yet for sessions yet, code is almost ready and needs to be adjusted/adapted a little bit.

-[ ] purge command sometimes responds with "unknown interaction", my guess is because it's taking longer than the secret discord limit for interactions which I think is 3 seconds.

-[ ] download-file command under duplicates the file extension (easy fix in `download.rs's` `file_to_attachment` method. Also needs  to prevent downloading files larger than 100mb, or upload them in chunks).
 