# Development Guide

## Linux Companion
```bash
cd linux
pytest -v tests
python3 -m build
```

## Android Application
```bash
cd android
./gradlew testDebugUnitTest
./gradlew assembleDebug
```

## Unified Build
```bash
./scripts/build-all.sh
```
