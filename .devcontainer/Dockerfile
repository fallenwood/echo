FROM mcr.microsoft.com/devcontainers/base:bookworm

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/debian.sources \
   && apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install mold pkg-config libssl-dev \
   && apt-get autoremove -y && apt-get clean -y

USER vscode
RUN curl https://sh.rustup.rs -sSf | sudo su - vscode -c "RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static bash -s -- -y -c rust-analyzer"
ENV PATH="/home/vscode/.cargo/bin:${PATH}"
