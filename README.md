[![docs-badge][]][docs] [![build badge]][build] [![guild-badge][]][guild] [![crates.io version]][crates.io link] [![rust 1.61.0+ badge]][rust 1.61.0+ link]

# Songbird

![](songbird.png)

Songbird is an async, cross-library compatible voice system for Discord, written in Rust.
The library offers:
 * A standalone gateway frontend compatible with [serenity] and [twilight] using the
 `"gateway"` and `"[serenity/twilight]-[rustls/native]"` features. You can even run
 driverless, to help manage your [lavalink] sessions.
 * A standalone driver for voice calls, via the `"driver"` feature. If you can create
 a `ConnectionInfo` using any other gateway, or language for your bot, then you
 can run the songbird voice driver.
 * And, by default, a fully featured voice system featuring events, queues, RT(C)P packet
 handling, seeking on compatible streams, shared multithreaded audio stream caches,
 and direct Opus data passthrough from DCA files.

## Intents
Songbird's gateway functionality requires you to specify the `GUILD_VOICE_STATES` intent.

## Codec support
Songbird supports all [codecs and formats provided by Symphonia] (pure-Rust), with Opus support
provided by [audiopus] (an FFI wrapper for libopus).

**By default, *Songbird will not request any codecs from Symphonia*.** To change this, in your own
project you will need to depend on Symphonia as well.

```toml
# Including songbird alone give you support for Opus via the DCA file format.
[dependencies.songbird]
features = ["builtin-queue"]

# To get additional codecs, you *must* add Symphonia yourself.
# This includes the default formats (MKV/WebM, Ogg, Wave) and codecs (FLAC, PCM, Vorbis)...
[dependencies.symphonia]
version = "0.5"
features = ["aac", "mp3", "isomp4", "alac"] # ...as well as any extras you need!
```

## Dependencies
Songbird needs a few system dependencies before you can use it.

- Opus - Audio codec that Discord uses.
If you are on Windows and you are using the MSVC toolchain, a prebuilt DLL is provided for you, you do not have to do anything.
On other platforms, you will have to install it. You can install the library with `apt install libopus-dev` on Ubuntu or `pacman -S opus` on Arch Linux.
If you do not have it installed it will be built for you. However, you will need a C compiler and the GNU autotools installed.
Again, these can be installed with `apt install build-essential autoconf automake libtool m4` on Ubuntu or `pacman -S base-devel` on Arch Linux.

This is a required dependency. Songbird cannot work without it.

- youtube-dl / yt-dlp / (similar forks) - Audio/Video download tool.
You can install the tool with Python's package manager, pip, which we recommend for youtube-dl. You can do it with the command `pip install youtube_dl`.
Alternatively, you can install it with your system's package manager, `apt install youtube-dl` on Ubuntu or `pacman -S youtube-dl` on Arch Linux.

This is an optional dependency. It allows Songbird to download audio/video sources from the Internet from a variety of webpages, which it will convert to the Opus audio format Discord uses.

## Examples
Full examples showing various types of functionality and integrations can be found in [this crate's examples directory].

## Contributing
If you want to help out or file an issue, please look over [our contributor guidelines]!

## Attribution
Songbird's logo is based upon the copyright-free image ["Black-Capped Chickadee"] by George Gorgas White.

[serenity]: https://github.com/serenity-rs/serenity
[twilight]: https://github.com/twilight-rs/twilight
["Black-Capped Chickadee"]: https://www.oldbookillustrations.com/illustrations/black-capped-chickadee/
[lavalink]: https://github.com/freyacodes/Lavalink
[this crate's examples directory]: https://github.com/serenity-rs/songbird/tree/current/examples
[our contributor guidelines]: CONTRIBUTING.md
[codecs and formats provided by Symphonia]: https://github.com/pdeljanov/Symphonia#formats-demuxers
[audiopus]: https://github.com/lakelezz/audiopus

[build badge]: https://img.shields.io/github/workflow/status/serenity-rs/songbird/CI?style=flat-square
[build]: https://github.com/serenity-rs/songbird/actions

[docs-badge]: https://img.shields.io/badge/docs-online-4d76ae.svg?style=flat-square
[docs]: https://serenity-rs.github.io/songbird/current

[guild]: https://discord.gg/9X7vCus
[guild-badge]: https://img.shields.io/discord/381880193251409931.svg?style=flat-square&colorB=7289DA

[crates.io link]: https://crates.io/crates/songbird
[crates.io version]: https://img.shields.io/crates/v/songbird.svg?style=flat-square

[rust 1.61.0+ badge]: https://img.shields.io/badge/rust-1.61.0+-93450a.svg?style=flat-square
[rust 1.61.0+ link]: https://blog.rust-lang.org/2022/05/19/Rust-1.61.0.html
