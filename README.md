# fricon

10 mK fridge controller.

## Development

### Requirements

* Stable Rust toolchain
* Protobuf compiler:

  ```console
  brew install protobuf # Mac
  pacman -S protobuf # Arch
  scoop install protobuf # Windows
  ```

* Sqlite3
* sqlx-cli

  ```console
  brew install sqlx-cli # Mac
  pacman -S sqlx-cli # Arch
  cargo install sqlx-cli
  ```

### Local setup

* `.env` file

  ```env
  DATABASE_URL=sqlite://.dev/testdb.sqlite3
  ```

* Local `.dev` folder should be ignored

  ```gitignore
  *
  ```

* Setup development database

  ```console
  sqlx db setup
  ```
