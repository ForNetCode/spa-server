# JS-APP-EXAMPLE
This is an example of React frontend project. The usage of `spa-client` is simple.

after bring `spa-server` up, just run `npm run build && npm run release`


## How to use with new project.
1. Install spa-client npm package.
```shell
npm install spa-client --save-dev
```
2. add configs for spa-client in the [.env](.env) file

3. Add script to package.json

```json
{
  "script":{
      "release":"dotenv .env.prod spa-client upload ./build www.example.com && spa-client release www.baidu.com"
  }
}
```
