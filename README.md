### General Info
I tried to do something with cryptography so I made my own encryption algorithm.
It's very weak compared to current AES or RSA but it's my first thing like that.

This program is terminal application so for more information just use `--help` argument.
But as basic information you give it files to encrypt (or decrypt) in arguments and it creates encrypted (or decrypted) copy of these files.
You can try adding `-V` option which shows progress bar.

### Building
This program doesn't use any of crates so it will build fastly.
To just try it out just run `cargo build --release` and move `target/release/dszru` binary to current directory or something like `~/.local/bin`.
