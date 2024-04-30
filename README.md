# **Crusty**

A simple discord music bot written in Rust.

## Functionalities

- [x] Playing from youtube url
- [x] Playing from youtube query
- [x] Nicely formatted embed messages
- [x] Seeking forward
- [x] Skipping songs
- [x] Queue
- [x] Playlist support
- [ ] New platforms support (Spotify, Soundcloud)
- [ ] Own queue implementation

## Commands

- play (aliases: p) [url | query] - plays video from url or searches for it with given query or adds it to queue
- join (revive) - joins the voice channel author of the message is currently in
- skip (fs, s) - skips the currently played song
- seek (forward) [secs] - skips specified number of seconds in the song 

## Requirements

The requirements are the same as for [songbird](https://github.com/serenity-rs/songbird/tree/current?tab=readme-ov-file#dependencies). You can skip the yt-dlp part if you don't want to have youtube playlist support.

## Installation

+ Download the repo via a git clone command or a zip source code
+ Add a DISCORD_TOKEN variable to your environment
+ Go to the project directory and run `cargo run -r` command

## Why Symphonia built from source?

Symphonia have problems with seeking forward in webm files for only a couple of seconds (perhaps a [bug](https://github.com/pdeljanov/Symphonia/issues/278))
so this version is fixed to support that.
