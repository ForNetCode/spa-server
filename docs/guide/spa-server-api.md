# HTTP API

Admin server provide http api to control static web files version upgrade, so should take care for access, it's disabled
by default.

The http api is described by `curl` command. You can run it in linux after variable changed.

## Authorization

It very simple, put `Token` to request http header: `Authorization: Bearer $TOKEN`

## Simple API Without `spa-client`

These api give you simple info about serving domain, and you can change the version of SPA.

There are environment variables for shell:

```shell
ADMIN_SERVER='http://127.0.0.1:9000'
# admin_server.token
TOKEN='token'
```

### Get all domains status

```shell
curl "$ADMIN_SERVER/status" -H "Authorization: Bearer $TOKEN"
# return json: [{"domain":"www.example.com","current_version":2,"versions":[1,2]}]
```

### Get specific domain status

```shell
DOMAIN='www.example.com'
curl "$ADMIN_SERVER/status?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return json: {"domain":"www.example.com","current_version":1,"versions":[1]} 
# or status code:404
```

### Get the domain upload file position info

it can be used with `rsync/scp` to upload web static files to the server.

```shell
FORMAT="Path"# or "Json", 
# "Path" will return the server location,you can use it with scp/rsync,
# "Json" will return the version and path
# format default value is "Path"
curl "$ADMIN_SERVER/upload/position?domain=$DOMAIN&format=$FORMAT" \
-H "Authorization: Bearer $TOKEN"
# return string if format is "Path": /$FILE_PATH/$DOMAIN/$NEW_VERSION 
# like /data/www.example.com/2
# return json if format is "Json": 
# {"path":"/$FILE_PATH/$DOMAIN/$VERSION", version:$VERSION, "status": $STATUS}
#
# $STATUS is a number,
# 0: there no domain in server,
# 1: the version directory is new,
# 2: the version directory already has file 
```

### Update the domain version

Please be attention:

**it will use the newest or biggest version after server restart/reload**

`OPT_VERSION` is optional, if not set, will try to use the max version of this domain to put it online.

```shell
OPT_VERSION=2

curl  -X POST "$ADMIN_SERVER/update_version"\
 -H "Authorization: Bearer $TOKEN" \
--data-raw `{
    "domain":$DOMAIN,
    "version": OPT_VERSION,    
}`
# return status code: 200(update version success)
# or 404 with string body: can not find files, please make sure you have upload files to correct place

# reload static web server
curl -X POST "$ADMIN_SERVER/reload" -H "Authorization: Bearer $TOKEN"
```

## Upload File API

These api are used with `spa-client` to upload files to the server.

### Get files metadata

The result is used to prepare for uploading file. and it will calculate all files md5, so it's slow when there are large
number of files.

```shell

curl "$ADMIN_SERVER/files/metadata?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
# return [{path:$path_string,md5:$md5_string, length: $file_length_integer}]
```

### Set domain version uploading status

`spa-client` use this api to tell admin server which domain version is to upload or upload finished.

```shell 
UPLOADING_STATUS=0 # Uploading:0, Finish:1

curl --location --request POST "$ADMIN_SERVER/files/upload_status" \
--header "Authorization: Bearer $TOKEN" \
--header 'Content-Type: application/json' \
--data-raw `{
    "domain":$DOMAIN,
    "version": $VERSION,
    "status": $UPLOADING_STATUS
}`
# return status code:200 if success 
```

### Upload file

The http body is `multipart/form-data` format.

```shell
PATH="/upload/file/path"
curl -X POST "$ADMIN_SERVER/file/upload?domain=$domain&version=$version&path=$PATH" \
-F "file=@$PATH" -H "Authorization: Bearer $TOKEN"
# return status code:200 if success 
```

### Delete deprecated domain files

```shell
# keep 2 versions. 
MAX_RESERVE_OPT = 1
curl -X POST "http://$ADMIN_SERVER/files/delete" \
 -H "Authorization: Bearer $TOKEN" \
--data-raw `{
  "domain":$DOMAIN,
  "max_reserve": $MAX_RESERVE_OPT
}`
# return status code:200 if success 
```

### Revoke version
**Attention: revoke version now is temp, when you reload or restart server, then It would use the max version.**
```shell
TARGET_VERSION=1
curl -X POST "$ADMIN_SERVER/files/revoke_version" \
 -H "Authorization: Bearer $TOKEN" \
--data-raw `{
  "domain":$DOMAIN,
  "version": $TARGET_VERSION
}`
```

### Get Cert version
```shell
curl -X POST "$ADMIN_SERVER/cert?domain=$DOMAIN_OPT" \
 - H "Authorization: Bearer $TOKEN"
```