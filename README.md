# Rust Robbo

Rust port of great 8-bit Atari game. It compiles to Web Assembly and is playable in [web browser](http://robbo.sed.pl/)

It uses graphics / level data from [GNU Robbo](http://gnurobbo.sourceforge.net)

![Robbo Screenshot](https://s3.eu-central-1.amazonaws.com/mrk-public/robbo/data/robbo-screenshot.png)

## Build instructions

### Prerequisites

* [rust](https://www.rust-lang.org/tools/install)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
* [basic-http-server](https://github.com/brson/basic-http-server)

```
$ wasm-pack build --target web -d www/pkg
$ basic-http-server ./www/
```

## Run

open [http://localhost:4000/](http://localhost:4000/) in your browser

### How to play

Move with arrows, shot with shift + arrow, reset level with Esc

Enyoy!
------
