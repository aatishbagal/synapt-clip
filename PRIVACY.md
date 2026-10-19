# Privacy Policy - SynaptClip

Last updated: October 2026

## What SynaptClip collects

SynaptClip does not collect any personal data. The application has no telemetry, no analytics, no crash reporting service, and no account system. Nothing is sent to any server operated by the developers, and nothing reaches the developers at all unless you choose to email a crash report (see Crash reports).

## Data stored on your device

SynaptClip stores the following data locally in a SQLite database on your device:

- Your clipboard history: the text content of items you copy while SynaptClip is running
- Pinned clips and any categories you create
- For clips received from a paired device, the sending device's name and identifier
- Application settings
- A crash log, if SynaptClip has crashed, kept until you send or dismiss the crash report prompt

This database is stored unencrypted in your user data directory and is readable by anything running as your user account.

This data never leaves your device except as described below.

## Clipboard access

SynaptClip reads your clipboard to build your history. It captures text only; images, files, and other clipboard formats are ignored.

Please read this section carefully:

**SynaptClip captures all text placed on the clipboard, including passwords, API keys, and other secrets.** It does not currently detect the markers that password managers use to flag a clipboard entry as concealed or transient, so a password copied from a password manager is stored in your clipboard history like any other text.

Captured text is stored locally and is never uploaded anywhere. You can delete individual clips, clear the entire history, or set an expiry period in Settings so clips are removed automatically after a chosen number of days.

## Network activity

Apart from update checks (described below), SynaptClip does not make any connections to the internet.

SynaptClip listens on the loopback address (127.0.0.1, port 57322) for clips forwarded by Synapt, and contacts Synapt on the loopback address to list paired devices. These connections never leave your machine.

When you explicitly send a clip to a paired device, SynaptClip hands the clip's text to Synapt, which transfers it to the target device over an encrypted connection on your local network. This only happens when you initiate it. Received clips are stored locally in your clipboard history and marked with the sender's device name.

### Update checks

SynaptClip checks GitHub for new versions of itself and of SynaptCore, the shared component it bundles. When automatic checks are on, a check runs 15 seconds after launch and each time you open Settings. A check also runs whenever you select Check for updates.

Each check is a standard HTTPS request to github.com, api.github.com, and raw.githubusercontent.com that identifies the app and its version. Like any web request, it reveals your IP address to GitHub. It contains no personal data, files, clipboard content, or device identifiers. When you choose to install an update, the installer is downloaded from GitHub Releases.

You can turn off automatic checks in Settings with the "Check for updates automatically" option. SynaptClip then contacts GitHub only when you select Check for updates.

Links in the app, such as "See all changes" in the release notes, open in your default web browser. SynaptClip does not fetch them itself.

## Crash reports

SynaptClip can detect when it has crashed during a previous session. If a crash is detected on the next launch, a prompt appears offering to send a report.

Crash reports are:

- Opt-in only: nothing is sent without your explicit action
- Sent via your own email client: SynaptClip opens a pre-filled draft and makes no network request of its own
- Addressed to synapt.reports@aatish.io
- Limited to the app version, the operating system, the time of the crash, the location in the code where the crash occurred, and the error message
- Shown to you in full before sending: you can read, edit, or discard the draft in your email client

SynaptClip does not add your files, clipboard content, or device identifiers to a crash report. The error message is produced by SynaptClip's own code and is not meant to contain personal data, but you can check it in the draft before you send it.

You can turn off crash report prompts at any time in Settings under Diagnostics, with the "Offer to email a crash report after a crash" option. The first-launch setup also asks whether to enable them.

## Permissions

SynaptClip requests the following system permissions:

- Clipboard access: to read and write clipboard content
- Auto-start (optional): if enabled in Settings, SynaptClip registers itself to start on login using the platform's standard mechanism (a systemd user service on Linux, a per-user Run registry value on Windows, a LaunchAgent on macOS)

SynaptClip does not require macOS Accessibility permission. Its global shortcut uses the Carbon hotkey API, which does not need it.

## Contact

For questions about this privacy policy, open an issue at https://github.com/aatishbagal/synapt-clip

## License

Copyright 2026 Aatish Bagal. Licensed under the Apache License, Version 2.0.
