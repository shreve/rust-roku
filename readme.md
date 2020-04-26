Roku Rust Remote
================

This library seeks out Roku devices on your network and controls them via Roku's ECP.

I wrote this as an exercise to learn Rust. I think [Peter Jacobs'
implementation](https://github.com/crespyl/rust-roku) is much better and safer.

## Usage

```rust
let client = Client::discover();
client.keypress("Up");
```
