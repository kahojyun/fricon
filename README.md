# fricon

Fridge controller.

## Documentation

[Github Pages](https://kahojyun.github.io/fricon/)

## Development

### Requirements

- Stable Rust toolchain
- Protobuf compiler:

    ```console
    brew install protobuf # Mac
    pacman -S protobuf # Arch
    scoop install protobuf # Windows
    ```

- Sqlite3
- sqlx-cli

    ```console
    brew install sqlx-cli # Mac
    pacman -S sqlx-cli # Arch
    cargo install sqlx-cli
    ```

### Local setup

- `.env` file

    ```env
    DATABASE_URL=sqlite://.dev/testdb.sqlite3
    ```

- Local `.dev` folder should be ignored

    ```gitignore
    *
    ```

- Setup development database

    ```console
    sqlx db setup --source crates/fricon/migrations
    ```

## License

`fricon` is distributed under the terms of the
[MIT](https://spdx.org/licenses/MIT.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html) license.
