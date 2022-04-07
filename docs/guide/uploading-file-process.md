# Uploading File Process 
This describes the process of `spa-client` uploading files to admin server.

Firstly, there are lots of files needed upload, so `retry` and `correct` must be considered.
there may exist large files which more than 20M(we would consider it later, we would have lots work to do to support 
resume breakpoint).

So the admin-server should provider api to show file metadata for checking file md5 and uploading status in server to avoid uploading conflict.

According to file metadata on the server side, `spa-client` could decide which file should be uploaded and which not,
this could save time when network errors happen.

Admin-server also needs to know which version is in the process of receiving files after restart. So when 
`spa-client` begin to upload file, it should tell admin-server, and admin-server add a file `.SPA-Processing` to
the directory `$Domain/$Version` if it not exists. admin-server will also reject the client uploading file if the
version is not set `.SPA-Prpccessing`. When `spa-client` tell admin-server uploading is finished, admin-server should
remove the file `.SPA-Processing`. The version which has `.SPA-Processing` should not be allowed to be online.


The above article do not consider how to deal with `S3` storage, we may later bring `S3` http client to admin-server
or `spa-client`, and do some work to improve the performance of `S3` files which are not cached in `spa-server`.