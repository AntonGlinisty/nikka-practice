FROM rust:1.63

RUN apt-get update
RUN apt-get install sudo qemu-system-x86 -y

RUN mkdir -p /opt/shad/rust
COPY .gitlab-ci.yml /opt/shad/.grader-ci.yml

WORKDIR /opt/shad/rust
COPY . .

RUN rm --force --recursive .git .jj
RUN rm --force --recursive $(find . -name \*.orig -o -name \*.rej -o -name \*.diff)

RUN make test
