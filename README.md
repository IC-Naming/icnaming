<div align="center">
<img src="asset/icnaming_logo.png" width="128" >
<h1>IC Naming</h1>
</div>

This repository contains the canister and canister-related components of IC Naming. 
You can own your web3 name with IC Naming.

## Development

Open `src` folder with Visual Studio Code with Remote Dev Tools extension, and load the source code in the container.

```shell
./sh_setup_dev.sh
./sh_go.sh
```

### Build on Windows

If you want to build this project on Windows, please install something below:

#### OpenSSL

install vcpkg <https://vcpkg.io/en/getting-started.html>

- ./vcpkg.exe install openssl-windows:x64-windows
- ./vcpkg.exe install openssl:x64-windows-static

set env:

OPENSSL_DIR="<vcpkg>\installed\x64-windows-static"

## Browser Extensions

You can find source code of the browser extension in the [icnaming-browser-extensions](https://github.com/IC-Naming/icnaming-browser-extensions) repository.

## Screenshots

![search_name](https://github.com/IC-Naming/icnaming-browser-extensions/releases/download/v0.1.0/search_name.gif)

![set_values](https://github.com/IC-Naming/icnaming-browser-extensions/releases/download/v0.1.0/set.gif)

![browser-extensions](https://github.com/IC-Naming/icnaming-browser-extensions/releases/download/v0.1.0/browser-extensions.gif)
