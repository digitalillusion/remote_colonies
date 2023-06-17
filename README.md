Remote Colonies
===============

A 2D RTS game written in Rust using Godot engine.

In this simple and addictive planetary conquest game, you build fleets on your home colony planet and you extract resources to build more ships. The more ships orbiting around a planet, the more the extraction rate.

However, sooner or later you’ll need to move your fleet and conquest other planets, in order to contrast the expansion of your adversaries.

Never move all your ships from a planet, though, or you will be an easy prey for the attacking enemy fleets.

Once that a player has conquered all the planets in the area, the game is over.

![Gameplay](https://i.ibb.co/B2fDw58/remote-colonies.gif "Gameplay")

## Build

Build the debug bindings for Godot 3.5

This will reference `remote_colonies_library_debug.gdnlib` in the project.

```shell
makers debug-godot
```

Build the release bindings for Godot 3.5

This will reference `remote_colonies_library.gdnlib` in the project.

```shell
makers release-godot
```

