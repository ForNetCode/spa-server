```shell
./create_self_signed_cert.sh --ssl-domain=local.fornetcode.com  --ssl-size=2048 --ssl-date=380

#openssl x509 -inform PEM -in local.fornetcode.com.crt -outform DER -out local.fornetcode.com.cer

# this is for apache/nginx
openssl x509 -in  local.fornetcode.com.crt -outform PEM -out local.fornetcode.com.pem
```