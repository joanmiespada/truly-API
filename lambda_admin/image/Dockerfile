FROM public.ecr.aws/lambda/provided:al2 as builder

RUN yum update -y && \
    yum groupinstall "Development Tools" -y && \
    yum install -y \
    yasm \
    nasm \
    wget \
    zlib-devel \
    libffi-devel \
    openssl-devel \
    openssl \
    bzip2-devel \
    libtool \
    glibc \
    glibc-utils \
    curl \
    ca-certificates

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /tmp

COPY lambda_admin/src /tmp/lambda_admin/src
COPY lambda_admin/Cargo.toml /tmp/lambda_admin/Cargo.toml
COPY Cargo.lock /tmp/lambda_admin/Cargo.lock
COPY lib_licenses /tmp/lib_licenses
COPY lib_users /tmp/lib_users
COPY lib_util_jwt /tmp/lib_util_jwt


WORKDIR /tmp/lambda_admin

RUN cargo build --release

RUN strip /tmp/lambda_admin/target/release/lambda_admin

FROM public.ecr.aws/lambda/provided:al2 

RUN yum install -y \
    openssl


WORKDIR /var/task

COPY --from=builder /tmp/lambda_admin/target/release/lambda_admin /var/runtime/bootstrap

CMD ["bootstrap.function_handler"]