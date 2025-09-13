FROM fluent/fluentd:v1.16-debian
USER root
RUN apt-get update && apt-get install -y netcat-openbsd && \
    gem install fluent-plugin-elasticsearch --no-document && \
    rm -rf /var/lib/apt/lists/*
USER fluent
