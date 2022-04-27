# spa-server distribution package
## Docker Image
The docker image is distributed at `Docker Hub` as `timzaak/spa-server`.

### AWS S3 Support
We support S3 storage by docker `panubo/docker-s3fs`, and release as `timzaak/spa-server:${version}-s3`, all configure about S3fs fuse can be found [here](https://github.com/panubo/docker-s3fs).

## From Code
There no plan to release binary package. You can `git clone` the code and build yourself.

```shell
git clone --recursive https://github.com/timzaak/spa-server
cargo build --package spa-server --release

# you could install it to your local server
cd server
cargo install --bin spa-server --path .
```
