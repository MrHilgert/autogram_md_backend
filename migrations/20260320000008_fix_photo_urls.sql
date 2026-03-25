-- Fix photo URLs from internal MinIO address to relative /photos/ path
UPDATE listing_photos
SET url = REPLACE(url, 'http://minio:9000/automarket-photos/', '/photos/')
WHERE url LIKE 'http://minio:9000/%';

UPDATE listing_photos
SET thumbnail_url = REPLACE(thumbnail_url, 'http://minio:9000/automarket-photos/', '/photos/')
WHERE thumbnail_url LIKE 'http://minio:9000/%';
