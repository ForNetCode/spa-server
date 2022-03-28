## What to do with SPA-Client
This doc describes why `spa-client` need to exist and the future of it.
### What problems it solves?
People would never try anything new if it looks same or worse to something they are family with and needs lots cost to try.
Comparing Nginx, `spa-server` do something better for static web server, but no 10x better than `nginx`. So there are rare
people would like to try it.

`spa-client` solve the above problem by providing an easy way to interact with `spa-server`.

Give people a try to use it replacing Nginx or other static web server by:

**One Command, Everything Is Ok!**

### How To Do
With this target, we should make a lot of tools.

* Command client `spa-client` for users of DevOps/Backend.
* `Vite` plugin wrapped `spa-client` to let users of frontend developers can integrate it with their project seamless.
* `Webpack` plugin wrapped `spa-client` to let users of frontend developers can integrate it with their project seamless.

After that, we also need to make binary of Linux/Mac/Window, and upload them to GitHub page to let users get it easily.

### Why use Rust
Maybe it may be good to use Typescript to integrate with `Vite` and `Webpack`. But the command line is also important.

### One future of `spa-client`
We will try to integrate `spa-client` with Nginx later. Maybe it's too hard to let people use `spa-server`.


## Design of `spa-client`
### configure
Configure should be allowed environment variables, files and command line options. There may have some configs about 
secret is not allowed to expose to environment variable.

Hocon format may not be a good choice for config, because people use `JSON` a lot in frontend area. and these plugins 
need to add extra package to parse Hocon format config file.

### Fast
fast is not a good feature, and people may do not need fast client with different reasons, we would provide 
config option about the parallel uploading size. 

### Interact with JS 
TODO:

### Others
Please ref [Uploading_File_Process.md](./Uploading_File_Process.md)



