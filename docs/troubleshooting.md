# Troubleshooting Guide

- **ADB device not recognized**:
  Ensure USB debugging is enabled and run `adb devices` on Linux to authorize the device.
- **Port already in use**:
  Specify an alternate port using `clink start --port 8780` and adjust the port in the Android settings.
- **Overlay click-through issues**:
  Ensure your X11 window manager supports compositing.
