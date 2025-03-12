# Based on https://github.com/SWI-Prolog/npm-swipl-wasm/blob/d2275193d38ffddb5f93828573f5363458d8c228/docker/Dockerfile
# Enables https://github.com/WebAssembly/exception-handling/blob/master/proposals/exception-handling/Exceptions.md
#   via https://emscripten.org/docs/tools_reference/settings_reference.html#wasm-exnref

FROM emscripten/emsdk:3.1.74-arm64

# Installs build dependencies.
RUN apt-get update && apt-get install -y ninja-build

# Installs a more up-to-date Node
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
RUN . ~/.nvm/nvm.sh && nvm install 22.9.0
RUN /usr/bin/versions/node/v22.9.0/bin/node --version

# Download dependency sources
WORKDIR /wasm
RUN wget --no-verbose -qO- https://zlib.net/zlib-1.3.1.tar.gz | tar xvz
RUN git clone --branch=pcre2-10.44 --depth 1 https://github.com/PCRE2Project/pcre2

# Build the dependencies and install them in /wasm
WORKDIR /wasm/zlib-1.3.1
RUN emconfigure ./configure --static --prefix=/wasm
RUN EMCC_CFLAGS="-Wno-deprecated-non-prototype -fno-exceptions -s WASM_EXNREF=1 -s SUPPORT_LONGJMP=wasm" emmake make
RUN emmake make install

WORKDIR /wasm/pcre2/build
RUN emcmake cmake -DCMAKE_INSTALL_PREFIX=/wasm \
  -DPCRE2GREP_SUPPORT_JIT=OFF \
  -G Ninja .. $@
RUN CFLAGS="-fno-exceptions -s WASM_EXNREF=1 -s SUPPORT_LONGJMP=wasm" ninja && ninja install

# Clone SWI-Prolog.  Only make a shallow clone and only clone the
# submodules we need at depth 1.
WORKDIR /
ARG SWIPL_COMMIT
RUN git clone --single-branch https://github.com/SWI-Prolog/swipl-devel

WORKDIR /swipl-devel
RUN git checkout f59d0715302daa640c58a4ca6a7ae232f25d1513
RUN git submodule update --init --depth 1 -j 100 \
  packages/chr packages/clib packages/clpqr packages/http packages/nlp \
  packages/pcre packages/plunit packages/sgml packages/RDF \
  packages/semweb packages/zlib

# Dodgy hack for experimental WASM features in Node
RUN echo '#!/bin/bash\nexec /usr/bin/versions/node/v22.9.0/bin/node --experimental-wasm-exnref "$@"' > /emsdk/node/20.18.0_64bit/bin/node
RUN chmod +x /emsdk/node/20.18.0_64bit/bin/node

# Build SWIPL
WORKDIR /swipl-devel/build.wasm
RUN mkdir -p /swipl-devel/build.wasm
RUN cmake -DCMAKE_TOOLCHAIN_FILE=/emsdk/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_FIND_ROOT_PATH=/wasm \
  -DUSE_GMP=OFF \
  -DINSTALL_DOCUMENTATION=OFF \
  -DCMAKE_C_FLAGS="-fno-exceptions -s WASM_EXNREF=1 -s SUPPORT_LONGJMP=wasm" \
  -G Ninja ..
RUN ninja

RUN ctest -j $(nproc) --stop-on-failure --output-on-failure
