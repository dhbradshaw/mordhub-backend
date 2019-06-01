docker stop mordhub-backend && docker rm mordhub-backend
docker run -it -p 3000:3000 --name mordhub-backend mordhub-backend:latest
