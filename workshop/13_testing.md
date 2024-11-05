# Testing

There are various ways to test a TUI application and it depends on how you structured your application. But in most cases, [testing with snapshots](https://ratatui.rs/recipes/testing/snapshots/) would be a viable solution.

> [!NOTE] 
> Switch to `chapter-13` branch to get ready for this chapter:
>
> ```sh
> git merge origin/chapter-13
> ```

Snapshots allow you to capture the output of your application and compare it with the expected output as shown below.

```log
running 1 test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ Snapshot Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Snapshot file: tui/src/snapshots/tui__tests__render_app.snap
Snapshot: render_app
Source: tui/src/main.rs:88
────────────────────────────────────────────────────────────────────────────────
Expression: terminal.backend()
────────────────────────────────────────────────────────────────────────────────
+new results
────────────┬───────────────────────────────────────────────────────────────────
          0 │+"┌[ Messages ]──────────────────────────────────────────────────┐┌[ Rooms ]─────┐"
          1 │+"│                                                              ││              │"
          2 │+"│                                                              ││              │"
          3 │+"│                                                              ││              │"
          4 │+"│                                                              ││              │"
          5 │+"│                                                              ││              │"
          6 │+"│                                                              ││              │"
          7 │+"│                                                              ││              │"
          8 │+"│                                                              ││              │"
          9 │+"│                                                              ││              │"
         10 │+"│                                                              ││              │"
         11 │+"│                                                              ││              │"
         12 │+"│                                                              ││              │"
         13 │+"│                                                              ││              │"
         14 │+"│                                                              ││              │"
         15 │+"│                                                              ││              │"
         16 │+"└──────────────────────────────────────────────────────────────┘└──────────────┘"
         17 │+"┌[ Send message () ]───────────────────────────────────────────────────────────┐"
         18 │+"│ Start typing...                                                              │"
         19 │+"└──────────────────────────────────────────────────────────────────────────[  ]┘"
────────────┴───────────────────────────────────────────────────────────────────
To update snapshots run `cargo insta review`
Stopped on the first failure. Run `cargo insta test` to run all snapshots.
test tests::test_render_app ... FAILED
```

For achieving this, we will be using the [insta](https://github.com/mitsuhiko/insta) crate.

If the output is different, the test will fail and you can review the changes with `cargo insta review`.

## Adding tests

First, add the `insta` crate as dev dependency (so that it only compiled when running tests):

```sh
cargo add insta --dev
cargo install cargo-insta # install the CLI tool
```

---

🎯 **Task**: Write a simple test case.

💡 **Tips:**

- See the following documentation:
  - [Rust unit testing](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)
  - [Ratatui backend method for retrieving te buffer](https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html#method.backend)
  - [insta's assert_snapshot](https://docs.rs/insta/latest/insta/macro.assert_snapshot.html)

<details>
<summary><b>Solution</b> ✅</summary>

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[tokio::test]
    async fn test_render_app() -> anyhow::Result<()> {
        let addr = Args::default();
        let addr = SocketAddr::new(addr.ip, addr.port);
        let mut app = App::new(addr);
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal.draw(|frame| app.draw_ui(frame))?;
        assert_snapshot!(terminal.backend());
        Ok(())
    }
}
```

</details>

---

When you run `cargo test`, the test will fail and it will generate a snapshot file in `src/snapshots`. You can _approve_ the snapshot by running `cargo insta review`.

Congratulations! You now have galaxy level tests 🛡️

---

> [!NOTE] 
> Get the completed code for this chapter by running:
>
> ```sh
> git merge origin/chapter-13-solution
> ```

<div style="text-align: right">

Continue to the [next chapter](./14_end.md) to wrap up the workshop. ➡️

</div>
