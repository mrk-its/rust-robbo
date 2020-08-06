./build.sh --release
# EXTRA_OPS='--dryrun'
COMMON_OPTS='--acl public-read --recursive'
set -x
aws s3 cp $EXTRA_OPTS $COMMON_OPTS --exclude '*' --include 'index.html' www ${S3_BUCKET}/robbo/
aws s3 cp $EXTRA_OPTS $COMMON_OPTS www/data/ ${S3_BUCKET}/robbo/data/
aws s3 cp $EXTRA_OPTS $COMMON_OPTS --exclude '*.wasm' www/pkg ${S3_BUCKET}/robbo/pkg/
aws s3 cp $EXTRA_OPTS $COMMON_OPTS --exclude '*' --include '*.wasm' --content-type 'application/wasm' --metadata-directive="REPLACE" www/pkg ${S3_BUCKET}/robbo/pkg/
