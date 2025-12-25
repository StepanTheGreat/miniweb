# miniweb 

Here I'm trying to experiment with tiny, `no_std` wasm binaries... by making a basic webgl game!

Because trying to achieve something without a specific goal in mind is quite confusing - I think
for a goal I'm going to try and make a super primitive space shooter. Nothing too fancy.

The objective would be to recreate the base game (spaceship, projectiles, score, aliens and so on), while at most using only rust code (interfacing with JS when needed). 

Let's start at 10K, and try to reduce it down to 4K. The ultimate target would be 2K (2048 bytes), but because my basic bump allocator and an array already take up ~820 bytes, this will be difficult.

Note that these numbers are **without** compression. With compression these will be much easier
to achieve, BUT, compression in itself also poses some difficulties when being loaded, hence we're not going to bother with it as of now.

## Instructions

A lot of basic cargo commands in this repo were transformed into special bash scripts.
To run them, you must have some tools installed, in particular:
- `build`: (`cargo`, [`binaryen`](https://github.com/WebAssembly/binaryen/tab=readme-ov-file#building))
- `serve`: `python3`
- `test`: (optionally `cargo-miri`)
- `bloat`: `cargo-twiggy`

Most of these are optional tools (all except for `build`) and you can avoid those entirely by
executing commands manually.

## P.S.
If this succeeds - in the future I'm going to redo this entire thing, but in android (with a different arcade game), but this time trying to build super-tiny apks.

I think these repos while useless in the real world, are super helpful at understanding how simple programs work under the hood. This might turn out to be a small `mini` series. 

## Licensing
Everything in this repo is licensed under MIT or Apache-2.0, under your choice.