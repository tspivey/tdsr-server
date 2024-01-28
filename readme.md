# tdsr server
This is a Windows server designed to take input from tdsr and output to NVDA.

## Usage
1. Download the NVDA controller client from https://www.nvaccess.org/files/nvda/releases/stable/
and extract it. You only need `nvdaControllerClient64.dll` and `nvdaControllerClient64.lib`.
2. Build it with `cargo build`, use `cargo run` or run `target\debug\tdsr-server`.
It will listen for connections on port 64111.

On the machine running tdsr, create a shell script, something like:
```shell
#!/bin/bash
exec socat - TCP4:SERVER_IP:64111,nodelay
```

Replace `SERVER_IP` with the IP address of the server.

## Security
This program binds to all interfaces on port 64111. The only things it lets you do are to have NVDA speak a string, and stop speech.

## Known bugs
1. This uses `BufReader::read_line` to read input. I don't know how to limit the amount of data it reads without abandoning it and doing my own data parsing,
so there's no limit on the length of a line that can be received.
