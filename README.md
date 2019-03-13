# Rust Robbo

Rust port of great 8-bit Atari game. It compiles to Web Assembly and is playable in [web browser](http://robbo.sed.pl/)

It uses graphics / level data from [GNU Robbo](http://gnurobbo.sourceforge.net)

![Robbo Screenshot](https://s3.eu-central-1.amazonaws.com/mrk-public/robbo/data/robbo-screenshot.png)

## Build instructions

### Prerequisites

* [rust](https://www.rust-lang.org/tools/install)
* [npm](https://www.npmjs.com/get-npm)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)


```
$ wasm-pack build
$ (cd pkg; npm link)
$ cd www
$ npm link rust-robbo
$ npm install
```

## Run
```
$ npm run start
```

open [http://localhost:8080/](http://localhost:8080/) in your browser (it should work on latest Chrome/Firefox)


### How to play

Move with arrows, shot with shift + arrow, reset level with Esc

Enyoy!
------
