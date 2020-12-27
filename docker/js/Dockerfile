FROM node:15.3.0-alpine3.11
RUN apk update -qq &&\
    apk add --no-cache build-base python
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.7.3/wait /wait
RUN chmod a+x /wait
RUN npm update -g npm &&\
    npm update -g yarn
