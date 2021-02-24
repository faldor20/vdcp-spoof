# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.158.0/containers/rust/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1


RUN rustup target add x86_64-pc-windows-gnu 
RUN rustup toolchain install nightly

RUN rustup target add x86_64-pc-windows-gnu --toolchain nightly

# [Optional] Uncomment this section to install additional packages.
 RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends mingw-w64 clang llvm-dev libxml2-dev uuid-dev libssl-dev bash patch make tar xz-utils bzip2 gzip sed cpio
#windows compiler
RUN ln -s windows.h /usr/x86_64-w64-mingw32/include/Windows.h && \
	ln -s shlobj.h /usr/x86_64-w64-mingw32/include/Shlobj.h