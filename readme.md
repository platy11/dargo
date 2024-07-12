# dargo

A program that allows you to use another device as a trackpad for your Linux
computer from a web browser.
It supports multitouch (e.g. two-finger scrolling and other gestures) and uses
Linux's uinput feature, so to other programs it looks just like any other input
device.

<!-- TODO: add screenshot -->

There is existing software that does pretty much the same thing, but I found
that a lot of them either weren't web-based or didn't support multitouch.
I also just thought it'd be an interesting project.

## Building

You'll need to have Rust/cargo and Node.js/npm installed.

```sh
make prepare
make
```

The compiled binary will be located at `dargo-server/target/release/dargo-server`.

You can supply the address to bind to as a command line argument, for example
`dargo-server 127.0.0.1:8080` (this address is also the default).

### Security considerations

By default, `dargo-server` binds to localhost:8080 - allowing connections
only from the same device (make sure you trust any other users of your
computer).
To securely allow other devices to connect, consider exposing `dargo-server` to
a VPN with only trusted devices, or running it behind an authenticating reverse
proxy.

## Known issues

- Firefox on Android doesn't seem to handle multiple simultaneous touches very
well
- Release/optimised builds with Rust 1.77/1.78 (and possibly other older
versions) don't work (gives the error `Invalid argument (os error 22)`)

## Future ideas

- Keyboard input?
- Media playback control via mpris/dbus?
