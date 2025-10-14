# Windows Troubleshooting Matrix

| Area | Symptom | First Steps |
|------|---------|-------------|
| Install | Installer fails to launch | Run `scripts/win/verify.ps1` and check SmartScreen | 
| Updates | Update rolls back | Collect updater logs and attach to issue |
| Git | Credential prompt missing | Ensure `win-askpass-pipe` feature enabled |
| Collaboration | Cannot connect behind proxy | Verify `HTTP_PROXY` settings |
