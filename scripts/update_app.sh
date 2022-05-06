#! /usr/bin/env bash
set -eo pipefail

# check if the doctl client is authenticated
# if ! [ "$(doctl auth list)" ]; then
#   echo >&2 "Error: doctl is not authenticated"
#   exit 1
# fi

SPEC_FILE="spec.yml"

# check if spec.yml exists
if ! [ -f "$SPEC_FILE" ]; then
  echo >&2 "Error: $SPEC_FILE could not be found."
  exit 1
fi

# get the name and id of the app in digital ocean
NAME_AND_APP_ID=($(doctl apps list --format "Spec.Name, ID" --no-header))

APP_ID="${NAME_AND_APP_ID[1]}"

echo "doctl app: $APP_ID updating..."

doctl apps update $APP_ID --spec=$SPEC_FILE --no-header --format 'ID'

echo "doctl app: $APP_ID updated!"
