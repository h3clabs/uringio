# syntax=docker/dockerfile:1

FROM ubuntu:24.04
SHELL ["/bin/bash", "-c"]

ARG SHELL
ARG WORKDIR
ARG DK_USER
ARG DEBIAN_FRONTEND=noninteractive

ENV LANG=en_US.utf8
ENV SHELL=${SHELL}

WORKDIR ${WORKDIR}
USER ${DK_USER}

COPY init.sh /init.sh
COPY entrypoint.sh /entrypoint.sh

RUN /init.sh

ENTRYPOINT ["/entrypoint.sh"]
