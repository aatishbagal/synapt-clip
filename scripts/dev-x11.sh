#!/bin/bash
# Launch SynaptClip in X11 mode via XWayland.
# Use this on GNOME Wayland sessions to develop against the arboard/X11 backend.
# Full Wayland support is added in v0.3.
GDK_BACKEND=x11 cargo tauri dev
