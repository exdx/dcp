# dcp: docker cp made easy

[![GitHub Actions](https://github.com/exdx/dcp/workflows/ci/badge.svg)](https://github.com/exdx/dcp/actions)
[![Latest version](https://img.shields.io/crates/v/dcp.svg)](https://crates.io/crates/dcp)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Summary

Containers are great tools that can encapsulate an application and its dependencies,
allowing apps to run anywhere in a streamlined way. Some container images contain
commands to start a long-lived binary, whereas others may simply contain data
that needs to be available in the environment (for example, a Kubernetes cluster).
For example, [operator-framework bundles](https://olm.operatorframework.io/docs/tasks/creating-operator-bundle/) and [crossplane packages](https://crossplane.io/docs/v1.9/concepts/packages.html) both use
container images to store Kubernetes manifests. These manifests are unpacked and applied to the cluster.

One of the downsides of using container images to store data is that they are
opaque. There's no way to quickly tell what's inside the image, although
the hash digest is useful in seeing whether the image has changed from a previous
version. The options are to use `docker cp` or something similar using podman
or containerd.

Using `docker cp` by itself can be cumbersome. Say you have a remote image
somewhere in a registry. You have to pull the image, create a container from that
image, and only then run `docker cp <container-id>` using an unintuitive syntax for selecting
what should be copied to the local filesystem.

dcp is a simple binary that simplifies this workflow. A user can simply
say `dcp <image-name>` and dcp can extract the contents of that image onto the
local filesystem. From there, users are free to view and edit the files locally. Any OCI-based image is supported. 

![Demo](demo.gif)

## Installing

### Installing from crates.io

If you're a Rust programmer and have Rust installed locally, you can install dcp
by simply entering `cargo install dcp`, which will fetch the latest version from
crates.io.
dcp relies on the stable Rust toolchain. 

### Download compiled binary

The [release section](https://github.com/exdx/dcp/releases) has a number
of precompiled versions of dcp for different platforms. Linux, macOS, and Windows (experimental)
binaries are pre-built. For MacOS, both arm and x86 targets are provided, and
for Linux only x86 is provided. If your system is not supported, building dcp from
the source is straightforward.

### Build from source

To build from source, ensure that you have the rust toolchain installed locally.
This project does not rely on nightly and uses the 1.62-stable toolchain.
Clone the repository and run `cargo build --release` to build a release version
of the binary. From there, you can move the binary to a folder on your $PATH to access
it easily.

## Implementation

Because there wasn't a suitable `containerd` client implementation in Rust at the time
of writing, dcp relies on APIs provided by external docker and podman crates. This limits dcp to working on systems where docker or podman is the container runtime.

By default, dcp will look for an active docker socket to connect to at the standard path. If the docker socket is unavailable, dcp will fallback to the current user's podman socket based on the $XDG_RUNTIME_DIR environment variable.

If the docker socket is on a remote host, or in a custom location, use the `-s` flag with the path to the custom socket.

## Flags and Examples

By default, dcp will copy content to the current working directory. For example, lets try issuing the following command:

```
$ dcp tyslaton/sample-catalog:v0.0.4 -c configs
```

This command will copy the `configs` directory (specified via the `-c` flag) from the image to the current directory.

For further configuration, lets try:

```
$ dcp tyslaton/sample-catalog:v0.0.4 -d output -c configs
```

This command pulls down the requested image, only extracting
the `configs` directory and copying it to the `output` directory
locally (specified via the `-d` flag). If `output` does not exist locally,
it will be created as part of the process. 

Another example, for copying only the manifests directory:

```
$ dcp quay.io/tflannag/bundles:resolveset-v0.0.2 -c manifests
```

Lastly, we can copy from a private image by providing a username
and password (specified via the `-u` and `-p` flags).

```
$ dcp quay.io/tyslaton/sample-catalog-private:latest -u <username> -p <password>
```

> :warning: This serves as a convenient way to copy contents from a private image
but is insecure as your registry credentials are saved in
your shell history. If you would like to be completely secure then
login via `<container_runtime> login` and pull the image first. dcp 
will then be able to find the image locally and process it.

## FAQ

**Q**: I hit an unexpected error unpacking the root filesystem of an image: `trying to unpack outside of destination path`. How can I avoid this?

**A**: dcp relies on the underlying `tar` Rust library to unpack the image filesystem represented as a tar file. The [unpack](https://docs.rs/tar/latest/tar/struct.Archive.html#method.unpack) method is sensitive in that it will not write files outside of the path specified by the destination. So things like symlinks will cause errors when unpacking. Whenever possible, use the `-c` flag to specify a directory to unpack, instead of the filesystem root, to avoid this error.

------------------
**Q**: I would like to use dcp to pull content from an image but I don't know where in the image the content is stored. Is there an `ls` command or similar functionality in dcp? 

**A**: Checkout the excellent [dive tool](https://github.com/wagoodman/dive) to easily explore a container filesystem by layer. After finding the path of the files to copy, you can then use dcp to extract just those specific files. 

------------------
**Q**: Is dcp supported on Windows?

**A**: Yes, dcp  is supported on Windows. Windows support is experimental, as there is no CI coverage, but it will likely work in your windows environment. The only non-default change you need to make is to expose the docker daemon so that dcp can connect to it. This can be done through one of two ways:

1. Adding the following to your `%userprofile%\.docker\daemon.json` file.
    ```json
    {
        "hosts": ["tcp://0.0.0.0:2375"]
    }
    ```

2. Going through the Docker Desktop UI and enabling the setting for `Expose daemon on tcp://localhost:2375 without TLS` under `General`.


------------------
**Q**: I would like to inspect image labels to figure out where in the filesystem I should copy from. Does dcp have an `inspect` command to list image labels?

**A**: Listing an image's labels can be done easily using the underlying container runtime. For example, run `docker image inspect <image-id> | grep Labels` to see labels attached to an image. From there, dcp can be used to copy files from the container filesystem. 

## Testing

If you would like to run the test suite, you just need to run the standard cargo command. This will run all relevant
unit, integration and documentation tests.

```
$ cargo test
```
