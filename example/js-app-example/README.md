# JS-APP-EXAMPLE
This is an example of React frontend project. The usage of `spa-client` is simple.

## How to run this example
```shell
# in project root directory
cd jsclient && npm install && npm run build
cd ../example/js-app-example && npm install && npm run build && npm run upload && npm run release
```
PS: `self.noti.link` is routed to 127.0.0.1

## How to use with new project
1. Install spa-client npm package.
```shell
npm install spa-client --save-dev
```
2. add configs for spa-client in the [.env](.env) file

3. Add script to package.json (need `dotenv` to inject config, you can also use config file as [SPA-Client](../../doc/SPA-Client.md)) said.

```json
{
  "script":{
      "upload": "dotenv .env.prod spa-client upload ./build www.example.com",
      "release":"dotenv .env.prod spa-client release www.baidu.com"
  }
}
```
