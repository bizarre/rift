# rift
###### [getting started](#getting-started) | [contributing](#contributing) | [issues](https://github.com/bizarre/rift/issues)
> Rift is a [fast](#fast) and simple layer 7 proxy for Minecraft servers heavily inspired by [BungeeCord](https://github.com/SpigotMC/BungeeCord) and [actix](https://github.com/actix).

## Disclaimer
This is not even close to being production ready, so don't use it in production. 

## Project Goals
- Support many protocol versions
- Support distributed proxy setups out the box
- Built-in web panel for analytics and configuration
- *Be very fast.. sonek fast ..*

## Getting Started
There's really not much you can do right now outside of manually building and running rift yourself.
###### Binaries will be provided once rift is production ready.

Clone and cd into rift directory
```console
git clone https://github.com/bizarre/rift.git && cd rift
```

Build and run rift
```console
cargo run
```
![img](https://i.imgur.com/YjVPbxU.png)

The default configuration binds to port `25570`, if you add `localhost:25570` to your Minecraft server list you should see **rift** running!

![img](https://i.imgur.com/xvfWy2Q.png)

That's about it. You can proceed to fine-tune the [config.toml](config.toml) to your liking.

## Contributing
You can start by trying to find an issue on the [issue tracker](https://github.com/bizarre/rift/issues). You can also just contribute by trying to use rift and reporting any issues you find. If you need help or want to have a deep conversation, send me an email at [alex@bizar.re](mailto:alex@bizar.re) or reach out on Discord (bizarre#0001).
