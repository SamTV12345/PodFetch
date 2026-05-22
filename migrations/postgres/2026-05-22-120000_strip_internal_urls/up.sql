UPDATE podcasts
SET image_url = ''
WHERE image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);

UPDATE podcasts
SET original_image_url = ''
WHERE original_image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);

UPDATE podcast_episodes
SET image_url = ''
WHERE image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);
