docker run --rm -v /tmp:/backup -v capsule-cache:/volume-data -w /volume-data busybox sh -c "tar -cvf /backup/capsule-cache.tar *"
