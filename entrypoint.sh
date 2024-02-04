#!/bin/sh

set -ex

echo "Generating assets"
/usr/bin/adbir

echo "Starting webserver"
exec darkhttpd /public
