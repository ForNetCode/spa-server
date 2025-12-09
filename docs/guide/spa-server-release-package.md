# spa-server distribution package
## Docker Image
The docker image is distributed at `Github Packages` as [`ghcr.io/fornetcode/spa-server`](https://github.com/fornetcode/spa-server/pkgs/container/spa-server).

```shell
docker pull ghcr.io/fornetcode/spa-server:latest
```
## From Code
```shell
git clone --recursive https://github.com/fornetcode/spa-server
cargo build --package spa-server --release

# you could install it to your local server
cd server
cargo install --bin spa-server --path .
```
