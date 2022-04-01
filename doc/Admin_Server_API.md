## Admin Server API

Admin server provide http api to control static web files version upgrade, be careful to it access config, and it's disabled by default.

The http api is described by `curl` command. You can run it with variable changed.
### Simple API Without `spa-client` 
these api give you simple info about the serving domain, and give you the control of domain version.
```shell
ADMIN_SERVER='http://127.0.0.1:9000'
TOKEN='token'

# get all domains status
curl "$ADMIN_SERVER/status" -H "Authorization: Bearer $TOKEN"
# return json: [{"domain":"www.example.com","current_version":1,"versions":[1]}]

# get specific domain status
DOMAIN='www.example.com'
curl "$ADMIN_SERVER/status?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return json: {"domain":"www.example.com","current_version":1,"versions":[1]} or status code:404

# get the domain upload file path, it can be used with `rsync/scp` to upload web static files to the server.
FORMAT="Path"# or "Json", 
# "Path" will return the server location,you can use it with scp/rsync,
# "Json" will return the version and path
# format default value is "Path"
curl "$ADMIN_SERVER/upload/position?domain=$DOMAIN&format=$FORMAT" -H "Authorization: Bearer $TOKEN"
# return string if format is "Path": /$FILE_PATH/$DOMAIN/$NEW_VERSION ,like /data/www.example.com/2
# return json if format is "Json": {"path":"/$FILE_PATH/$DOMAIN/$VERSION", version:$VERSION, "status": $STATUS}
# $STATUS is a number, 0: there no domain in server, 1:the version directory is new, 2: the version directory already has file 

# update the domain version. please be attention:
# *it will use the newest version after server restart/reload*
# *it will use the newest version after server restart/reload*
# *it will use the newest version after server restart/reload*
OPT_VERSION=2 # version is optional, if not set, will try to use the max version of this domain to put it online.
curl  -X POST "$ADMIN_SERVER/update_version" -H "Authorization: Bearer $TOKEN" \
--data-raw `{
    "domain":$DOMAIN,
    "version": OPT_VERSION,    
}`
# return status code: 200(update version success) or 404(can not find files, please make sure you have upload files to correct place)

# reload static web server
curl -X POST "$ADMIN_SERVER/reload" -H "Authorization: Bearer $TOKEN"
```

### Uploading File API(with `spa-client`)
These api are used with `spa-client` to upload files to the server. the api design is described in the doc 
[Uploading_File_Process.md](design/Uploading_File_Process.md)

```shell
# get files metadata to prepare to upload file.
# this api will calculate all files md5, so it's slow when there are large number of files.
curl "http://$ADMIN_SERVER/files/metadata?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
# return [{path:$path_string,md5:$md5_string, length: $file_length_integer}]


# set the domain version uploading status
UPLOADING_STATUS=0 # Uploading:0, Finish:1

curl --location --request POST "http://$ADMIN_SERVER/files/upload_status" \
--header "Authorization: Bearer $TOKEN" \
--header 'Content-Type: application/json' \
--data-raw `{
    "domain":$DOMAIN,
    "version": $VERSION,
    "status": $UPLOADING_STATUS
}`
# return status code:200 if success 

# upload file
PATH="/upload/file/path"

curl -X POST "http://$ADMIN_SERVER/file/upload" \
-F "file=@$PATH" -F "domain=$DOMAIN" -F "version=$VERSION" -F "path=$PATH"  -H "Authorization: Bearer $TOKEN"
# return status code:200 if success 
```
