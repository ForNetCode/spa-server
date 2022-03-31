# JS-APP-EXAMPLE
This is a React frontend project. The usage of `spa-client` is simple.

## Usage
```shell
npm install spa-client
```
add script to package.json, and all configs for spa-client are in the [.env](.env) file

```json
{
  "script":{
      "release":"dotenv .env.prod spa-client upload ./build www.example.com && spa-client release www.baidu.com"
  }
}
```
