

## GFX SSL v2 SDK


### Jupiter Integration
By default, the SDK compiles with the `jupiter_amm` feature.

This feature requires that you point it to a dylib.

There is a `jupiter_quote` example in `examples/jupiter_quote.rs`.
You can run it the example with:
```
LD_LIBRARY_PATH="./lib/linux/x86_64" cargo run --example jupiter_quote -p gfx-ssl-v2-sdk
```
