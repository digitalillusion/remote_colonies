Remote Colonies
===============

A 2D RTS game written in Rust using Godot engine.

In this simple and addictive planetary conquest game, you build fleets on your home colony planet and you extract resources to build more ships. The more ships orbiting around a planet, the more the extraction rate.

However, sooner or later you’ll need to move your fleet and conquest other planets, in order to contrast the expansion of your adversaries.

Never move all your ships from a planet, though, or you will be an easy prey for the attacking enemy fleets.

Once that a player has conquered all the planets in the area, the game is over.

## Play 

You can play the game online from https://deverse.xyz/project/remote-colonies

![Gameplay](https://i.ibb.co/B2fDw58/remote-colonies.gif "Gameplay")


### Instructions

The planets have a label indicating respectively the amount of resources available and the amount of resources extracted.
Every 10 resources extracted it's possible to build a new ship which will begin orbiting the planet by a left mouse button click on it.

The more ships orbiting a planet, the quicker resources get extracted. In order to move ships from a planet to another, left mouse button click on the departure planet and drag onto the destination planet.
Half of the ships available on the departure planet will be moved.

If there are ships belonging to several players on a planet, they will battle: there will be repeated fights and at each fight one ship will be killed. The probability to win a fight is proportional to the amount of ships of the same player remaining on the planet.

The game ends when ships of one player are the only remaining on the board, and he will be winner. In the case a player has no more ships remaining on the board, he is eliminated.

## Build

The game requires [Godot engine 3.x](https://godotengine.org/download/3.x/) to run, in order to produce a binary and to build the HTML5 executable. 
It uses [gdnative](https://docs.godotengine.org/it/stable/tutorials/scripting/gdnative/what_is_gdnative.html) to callback on the `gdnlib` dynamic library produced by the Rust build, according to the build target:

### Debug bindings

```shell
makers debug-godot
```

This command will change the reference toward `remote_colonies_library_debug.gdnlib` inside the Godot project.

### Release bindings

```shell
makers release-godot
```

This command will change the reference toward `remote_colonies_library.gdnlib` inside the Godot project.

### WASM bindings

Alternatively, it's possible to build an [HTML5 release](https://godot-rust.github.io/book/gdnative/export/html5.html).

```shell
makers release-html5
```

This command will build the WASM bindings; it *requires* Rust nightly-2023-01-27 and Emscripten 3.1.21 

In order succeed, *the godot template must be built first*, as described in the gdnative export documentation above.

After generating the wasm file, it's possible to export the project as HTML5 using Godot engine.

## References

[mcts crate](https://docs.rs/mcts) was integrated in the project in order to build WASM target, which requires a single thread