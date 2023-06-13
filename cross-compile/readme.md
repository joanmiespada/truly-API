# OpenSSL dependency

Bootstrap Docker image for cross-compilation:

* aarch64-linux-gnu: graviton AWS Lambda architectures
* aarch64-mac: M1 docker images, local development
* x86_64-linux-gnu: AWS Lambdas

## aarch64

Folder: ***aarch64-linux-gnu***

```bash
docker build -t cross-compile-environment .
docker run -it cross-compile-environment
```

Jump into docker container:

```bash
cd home
wget https://www.openssl.org/source/openssl-xxxxx.tar.gz
tar -xvf openssl-xxxxxx.tar.gz
cd openssl-xxxxx -xvf openssl-xxxxx.tar.gz
cd openssl-xxxx
```

Compile ssl

```bash
./Configure linux-aarch64 --cross-compile-prefix=aarch64-linux-gnu-
make
```

Once it has been compiled successfully, last but not least, copy artifacts to your host:

include folder:

```bash
docker cp <containerId>:/home/openssl-3.1.1/include .
```

lib folder:

```bash
# inside the container
cd lib
libcrypto.a
libcrypto.ld
libcrypto.pc
libcrypto.so
libcrypto.so.3
libssl.a
libssl.ld
libssl.pc
libssl.so
libssl.so.3

mkdir artifacts
mv libcrypto.* artifacts
mv libssl.* artifcats
```

```bash
#host
docker cp <containerId>:/home/openssl-3.1.1/artifacts .
mv artifacts lib
```

## x86_64

Folder: ***x86_64-linux-gnu***

Just friendly remainder about rustup:

```bash
rustup target add x86_64-unknown-linux-gnu
```

```bash
docker build -t cross-compile-environment .
docker run -it cross-compile-environment
```

```bash
./Configure linux-x86_64 --cross-compile-prefix=x86_64-linux-gnu-
make
```

You can copy headers and libs as we did it above.
