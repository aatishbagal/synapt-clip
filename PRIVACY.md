# Privacy Policy - SynaptClip

Last updated: August 2026

## What SynaptClip collects

SynaptClip does not collect any personal data. The application has no telemetry, no analytics, no crash reporting service, and no account system. Nothing is sent to any server operated by the developers.

## Data stored on your device

SynaptClip stores the following data locally in a SQLite database on your device:

- Your clipboard history: the text content of items you copy while SynaptClip is running
- Pinned clips and any categories you create
- For clips received from a paired device, the sending device's name and identifier
- Application settings

This database is stored unencrypted in your user data directory and is readable by anything running as your user account.

This data never leaves your device except as described below.

## Clipboard access

SynaptClip reads your clipboard to build your history. It captures text only; images, files, and other clipboard formats are ignored.

Please read this section carefully:

**SynaptClip captures all text placed on the clipboard, including passwords, API keys, and other secrets.** It does not currently detect the markers that password managers use to flag a clipboard entry as concealed or transient, so a password copied from a password manager is stored in your clipboard history like any other text.

Captured text is stored locally and is never uploaded anywhere. You can delete individual clips, clear the entire history, or set an expiry period in Settings so clips are removed automatically after a chosen number of days.

## Network activity

SynaptClip does not make any connections to the internet.

SynaptClip listens on the loopback address (127.0.0.1, port 57322) for clips forwarded by Synapt, and contacts Synapt on the loopback address to list paired devices. These connections never leave your machine.

When you explicitly send a clip to a paired device, SynaptClip hands the clip's text to Synapt, which transfers it to the target device over an encrypted connection on your local network. This only happens when you initiate it. Received clips are stored locally in your clipboard history and marked with the sender's device name.

## Permissions

SynaptClip requests the following system permissions:

- Clipboard access: to read and write clipboard content
- Auto-start (optional): if enabled in Settings, SynaptClip registers itself to start on login using the platform's standard mechanism (a systemd user service on Linux, a per-user Run registry value on Windows, a LaunchAgent on macOS)

SynaptClip does not require macOS Accessibility permission. Its global shortcut uses the Carbon hotkey API, which does not need it.

## Contact

For questions about this privacy policy, open an issue at https://github.com/aatishbagal/synapt-clip

## License

Copyright 2026 Aatish Bagal. Licensed under the Apache License, Version 2.0.
