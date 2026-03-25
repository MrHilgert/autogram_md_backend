-- Migrate photo URLs from local /photos/ path to Cloudflare R2 CDN
UPDATE listing_photos
SET url = REPLACE(url, '/photos/', 'https://cdn.car.hilgert.cc/')
WHERE url LIKE '/photos/%';

UPDATE listing_photos
SET thumbnail_url = REPLACE(thumbnail_url, '/photos/', 'https://cdn.car.hilgert.cc/')
WHERE thumbnail_url LIKE '/photos/%';
