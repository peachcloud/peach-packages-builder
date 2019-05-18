# debian-packages-builder

Server to automatically build [`peach-packages`](https://github.com/peachcloud/peach-packages) when `release` branch is pushed.


TODO:
- [x] Runs an HTTP server to listen to GitHub Web Hooks and build when packages are released.
- [ ] Package `debian-packages-builder` as a Debian package
  - [ ] Get GitHub Web Hook secret from user config
  - [ ] On install, setup schroot
  - [ ] On remove, clean schroot
  - [ ] On install, add hook server as Systemd service
  - [ ] On install, add two Nginx configs
      - [ ] Proxy to hook server
      - [ ] Host resulting apt files in `./packages/repo`
  - [ ] On remove, clean Nginx configs
- [ ] Investigate sending build status back to GitHub
