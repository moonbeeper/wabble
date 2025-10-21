# wabble

> For Siege Week 3! Themed around signals

Wabble is a typical clone of *Pictochat*... and that's it. There's not much added to it, everything is only stored in memory and nothing is persisted. You even have Private Rooms, which are not shared with anyone else besides you or people that you gave the room code to, for sending those nice cat or avali drawings.

Wabble is divided into two parts: The Rust server and the Godot client (which is a bit of a mess, but is sure does the job).

## Running

### Backend

You have two options, using the backend hosted on [Nest](https://hackclub.app/) or running it yourself. I am using the former for the demo on itch.io which you can find... [on itch.io](https://moonbeeper.itch.io/wabble). BUT, if you want to run the backend yourself, you can do so by following the tiny steps below.

First, to run the backend you need to have (obviously) [Rust](https://rust-lang.org/learn/get-started/) installed. After that, we first generate the `settings.toml` file for the server, which provides a simple way of configuring the bind address of the http server. It **is** overkill for this, but it's better than using environment variables. (If you want to use environment variables, prefix the variables with `WAB`). So, run:

```sh
cd wabble_server

cargo run -- -g
```

By running this, you have also downloaded the dependencies for the server! That means, you can now run the server with:

```sh
cargo run
```

### Godot

Just like the backend, you can run the Godot client from the (godot) editor or use the demo on [on itch.io](https://moonbeeper.itch.io/wabble).

If you wanted to run the client on the editor, I am using the v4.5.stable.official version of Godot. Aaaand, that's it open the project, click play and you're good to go!

Remember that you can switch the websocket address in the `scripts/game_manager.gd` file if you want to use a different server than the one I am hosting.
