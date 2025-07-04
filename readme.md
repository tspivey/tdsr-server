# tdsr server
This is a Windows server designed to take input from tdsr and output to NVDA, JAWS, or any other screen reader/TTS.

## Usage
Build it with `cargo build --release` and run `target\debug\tdsr-server`. It will listen for connections on port 64111 by default, but you can specify a different port as a command-line argument.

On the machine running tdsr, create a shell script, something like:
```shell
#!/bin/bash
exec socat - TCP4:SERVER_IP:64111,nodelay
```
Replace `SERVER_IP` with the IP address of the server, and 64111 with your custom port, if you specified one.

The server spawns a simple system tray icon. Right clicking on it will allow you to cleanly exit the server.

## Security
This program binds to all interfaces on port 64111. The only things it lets you do are to have TTS speak a string and stop speaking.
