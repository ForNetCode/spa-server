## Admin Server API

Admin server provide http api to control static web files version upgrade, be careful to it access config, and it's disabled by default.

The http api is described by `curl` command. You can run it with variable changed.
### Simple API before V1.2.3
these api give you simple info about the serving domain, and give you the control of domain version.
```shell
ADMIN_SERVER='127.0.0.1:9000' 
TOKEN='token'

# get all domains status
curl "http://$ADMIN_SERVER/status" -H "Authorization: Bearer $TOKEN"
# return json: [{"domain":"www.example.com","current_version":1,"versions":[1]}]

# get specific domain status
DOMAIN='www.example.com'
curl "http://$ADMIN_SERVER/status?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return json: {"domain":"www.example.com","current_version":1,"versions":[1]} or status code:404

# get the domain upload file path, it can be used with `rsync/scp` to upload web static files to the server.
curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return string: /$FILE_PATH/$DOMAIN/$NEW_VERSION ,like /data/www.example.com/2

# update the domain version. please be attention:
# *it will use the newest version after server restart/reload*
# *it will use the newest version after server restart/reload*
# *it will use the newest version after server restart/reload*
VERSION=2
curl  -X POST "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
# return status code: 200(update version success) or 404(can not find files, please make sure you have upload files to correct place)

# reload static web server
curl -X POST "http://$ADMIN_SERVER/reload" -H "Authorization: Bearer $TOKEN"
```

### Uploading File API
These api are used with `spa-client` to upload files to the server. the api design is described in the doc 
[Uploading_File_Process.md](./Uploading_File_Process.md)

```shell
# get files metadata to prepare to upload file.
curl "http://$ADMIN_SERVER/files/metadata?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
# return TODO


# set the domain version uploading status
UPLOADING_STATUS=0 # Uploading:0, Finish:1

curl --location --request POST "http://$ADMIN_SERVER/files/upload_status" \
--header "Authorization: Bearer $TOKEN" \
--header 'Content-Type: application/json' \
--data-raw `{
    "domain":$DOMAIN,
    "version": 2,
    "status": $UPLOADING_STATUS
}`
# return status code:200 if success 

# upload file
PATH="/upload/file/path"
curl -X POST "http://$ADMIN_SERVER/file/upload" \
-F "file=@$PATH" -F "domain=$DOMAIN" -F "version=$VERSION" -F "path=$PATH"  -H "Authorization: Bearer $TOKEN"
```
