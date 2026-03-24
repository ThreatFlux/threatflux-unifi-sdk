FROM rust:1.94-bookworm

ENV DEBIAN_FRONTEND=noninteractive

ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid ${USER_GID} ${USERNAME} \
    && useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME} \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        sudo \
        ca-certificates \
        apt-transport-https \
        wget \
        gnupg \
        lsb-release \
        curl \
    && echo "${USERNAME} ALL=(root) NOPASSWD:ALL" >/etc/sudoers.d/${USERNAME} \
    && chmod 0440 /etc/sudoers.d/${USERNAME} \
    && rm -rf /var/lib/apt/lists/*

RUN wget -qO - https://pgp.mongodb.com/server-7.0.asc | gpg --dearmor -o /etc/apt/trusted.gpg.d/mongodb-server-7.0.gpg \
    && echo "deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/debian bookworm/mongodb-org/7.0 main" > /etc/apt/sources.list.d/mongodb-org-7.0.list \
    && wget -O /etc/apt/trusted.gpg.d/unifi-repo.gpg https://dl.ui.com/unifi/unifi-repo.gpg \
    && echo "deb [ arch=amd64,arm64 ] https://www.ui.com/downloads/unifi/debian stable ubiquiti" > /etc/apt/sources.list.d/100-ubnt-unifi.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends mongodb-org-server unifi \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8443 8080 3478/udp 10001/udp 6789

USER ${USERNAME}

WORKDIR /workspaces/threatflux-unifi-sdk
