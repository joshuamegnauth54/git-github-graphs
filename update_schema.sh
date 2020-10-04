#!/bin/sh

ERRORCODE=-1
QUERY_DIR=queries
URL="https://raw.githubusercontent.com/octokit/graphql-schema/master/schema.graphql"
USERAGENT="RepoScraper GraphQL"

cd $QUERY_DIR || {
    echo "Failed to change to the query directory: $QUERY_DIR"
    exit $ERRORCODE;
}

curl --user-agent "$USERAGENT" -o ghschema.graphql "$URL"

exit 0
