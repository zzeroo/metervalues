#!/bin/sh

dir="$(basename "$PWD")"
zip -r "../${dir}.zip" . -x "./target/*" "./.git/*" "./.env" "./.env.*"
