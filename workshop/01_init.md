# Initialization

Here is your first task! Create a new project named "tui" in the workspace.

If you are already familiar with `cargo`, this should be a breeze. If not, don't worry! Just check the output of `cargo --help`.

<details>
<summary><b>Solution</b> ✅</summary>

```sh
cargo new tui
```

This will create a new directory named `tui` with the following structure:

```
tui
├── Cargo.toml
└── src
    └── main.rs
```

`tui/Cargo.toml` is the manifest file for the project where we will be defining the dependencies and other configurations. Go and take a look at it :)

Also, the top-level `Cargo.toml` in the workspace should now look like this:

```toml
[workspace]
resolver = "2"
members = ["common", "server", "tui"]
```

</details>

Since our new `tui` project is the member of the workspace now, we can try running it via:

```sh
cargo run -p tui
```

If everything goes right, you should see the output `Hello, world!` in the terminal. Voila! You have just created a Rust project.

But don't get too excited, we have a long way to go.

---

> [!NOTE] 
> Get the initial code for the TUI project by running:
>
> ```sh
> git merge origin/chapter-1
> ```

<div style="text-align: right">

Continue to the [next chapter](./02_tui.md) to initialize TUI with Ratatui. ➡️

</div>
