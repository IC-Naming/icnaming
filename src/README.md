
## Ready to Go

open this directory with Visual Studio Code and "Open in Container"

```shell
./sh_setup_dev.sh
./sh_go.sh
```

## Build on Windows

If you want to build this project on Windows, please install something below:

### OpenSSL

install vcpkg <https://vcpkg.io/en/getting-started.html>

- ./vcpkg.exe install openssl-windows:x64-windows
- ./vcpkg.exe install openssl:x64-windows-static

set env:

OPENSSL_DIR="<vcpkg>\installed\x64-windows-static"