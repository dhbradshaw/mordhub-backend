docker stop my_postgres && docker rm my_postgres
docker stop mordhub-backend && docker rm mordhub-backend
docker network create mynet
docker run -d -p 5432:5432 -e POSTGRES_USER=admin -e POSTGRES_PASSWORD=Password1 -e POSTGRES_DB=mordhub --network mynet --name my_postgres postgres:alpine
docker build -t mordhub-backend .
docker run -p 3000:3000 --network=mynet --name mordhub-backend  mordhub-backend:latest
