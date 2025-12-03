---
sidebarDepth:2
---

# What is spa-server?

spa-server is to provide a static web http server with cache and hot reload.
It supports multiple configs for different domains, and has a client tool(npm package, command line) to help upload
static web files to server.

::: info Need Feedback
sap-server features have been done, we are willing to get your feedback, fell free to
open [issues](https://github.com/fornetcode/spa-server/issues).
:::

## Motivation

In my company, every single page application needs a nginx docker image, as time-long, these containers take lots of
resources of memory and storage, and these nginx don't have a proper config.

So I tried to develop a static web server to solve the above problem, and create a client tool `spa-client` to help
users to release SPA.

## Feature

- Built with Salvo, fast and small!
- Static web version control, you can regress or release a new version easily.
- Docker support(compressed size: 32M).
- Provide command line/npm package to deploy spa.
- Multiple configs for different domains and Multiple SPA in on domain.
