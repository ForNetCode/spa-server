# SPA-Client
this doc provides all info about spa-client config and instructions for use.


## Shell Command
the config file contains information about the admin server and some default options.
its path: `config_path` default value is `./client_config.conf`, and some options could be obtained from environment variable.

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

## Config
### Config Load Order
command line option > environment  > `config-dir` file
