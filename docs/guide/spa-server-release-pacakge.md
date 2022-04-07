# spa-server distribution package
## Docker Image
The docker image is distributed at `Docker Hub` as `timzaak/spa-server`, it support `linux/amd64`,`linux/arm64`.

## From Code
There no plan to release binary package. You can `git clone` the code and build yourself.

```shell
git clone --recursive https://github.com/timzaak/spa-server
cargo build --package spa-server --release

# you could install it to your local server
cd server
cargo install --bin spa-server --path .
```
