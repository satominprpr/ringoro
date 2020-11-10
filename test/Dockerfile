FROM ruby:2.7.2-alpine
RUN apk update -qq &&\
    apk add --no-cache build-base libffi-dev
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.7.3/wait /wait
RUN chmod a+x /wait
RUN mkdir /work
WORKDIR /work
ENV BUNDLE_PATH /work/.bundle
