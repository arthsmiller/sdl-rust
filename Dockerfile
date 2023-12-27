FROM rust:latest

RUN apt-get update && apt-get install -y \
    libsdl2-2.0-0 libsdl2-dev \
    libsdl2-ttf-2.0-0 libsdl2-ttf-dev \
    libsdl2-gfx-1.0-0 libsdl2-gfx-dev

#ENV XDG_RUNTIME_DIR /run/user/$(id -u)
