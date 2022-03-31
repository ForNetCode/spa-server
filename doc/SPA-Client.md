# SPA-Client
this doc provides all info about spa-client config and instructions for use. And first of all `spa-server` should open admin-server http api, then
`spa-client` could interact with it.

`spa-client` include commandline and js package two parts, js package wraps the commandline 
by [napi-rs](https://github.com/napi-rs/napi-rs) like [SWC](https://github.com/swc-project/swc).
So both have same user experience and same api. There also has a example project for js package users: [js-app-example](../example/js-app-example).

## How to use commandline of `spa-client`
you need to set config by `config-dir` option or environment variables like [.env](../example/js-app-example/.env). Config load order is
command line option > `config-dir` file > environment.

There are some usage examples of `spa-client`, you also can get help by typing `spa-client -h`.
```shell
# upload static files to admin server, if not set $OPT_VERSION, will try to 
spa-client --config-dir $CONFIG_PATH upload $STATIC_FILE_PATH $DOMAIN $OPT_VERSION -p 3

# tell admin server to release the specific domain version to public. if don't set $OPT_VERSION, will fetch the max version to be online, if the max version it under uploading process, release will fail. 
spa-client --config-dir $CONFIG_PATH release $DOMAIN $OPT_VERSION

# get info of the specific domain or all domain, just like the admin server http api.
spa-client --config-dir $CONFIG_PATH info $OPT_DOMAIN

# reload spa-server
spa-client --config-dir $CONFIG_PATH reload
```
You could use `upload` and `relase` commands to release new version. If you don't want use `spa-client` to upload files, you could do it like this:

```shell
scp $SPA_DIRECTORY user@ip:$(curl "http://$ADMIN_SERVER/upload/position?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN") && \
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
```

All the admin server http api is in the [Admin_Server_API.md](./doc/Admin_Server_API.md).
