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
container images to store Kubernetes manifests. These manifests are unpacked on-cluster and made available to end users.

One of the downsides of using container images to store data is that they are
opaque. There's no way to quickly tell what's inside the image, although
the hash digest is useful in seeing whether the image has changed from a previous
version. The options are to use `docker cp` or something similar using podman
or containerd.

Using `docker cp` by itself can be cumbersome. Say you have a remote image
somewhere in a registry. You have to pull the image, create a container from that
image, and only then run `docker cp <container-id>` using an unintuitive syntax for selecting
what should be copied to the local filesystem.

dcp is a simple binary that attempts to simplify this workflow. A user can simply
say `dcp <image-name>` and it can extract the contents of that image onto the
local filesystem. It can also just print the contents of the image to stdout, and
not create any local files.

![Demo](demo.gif)

## Installing

### Installing from crates.io

If you're a Rust programmer and have Rust installed locally, you can install dcp
by simply entering `cargo install dcp`, which will fetch the latest version from
crates.io.

### Download compiled binary

The [release section](https://github.com/exdx/dcp/releases) has a number
of precompiled versions of dcp for different platforms. Currently only Linux and
MacOS are pre-built. For MacOS, both arm and x86 targets are provided, and
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

## Flags and Examples

By default, dcp will copy content to the current directory `.`. For example, lets
try issuing the following command:

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
locally (specified via the `-d` flag).

Another example, for copying only the manifests directory:

```
$ dcp quay.io/tflannag/bundles:resolveset-v0.0.2 -c manifests
```

Lastly, we can reference a private registry by providing a username
and password (specified via the `-u` and `-p` flags).

```
$ dcp quay.io/tyslaton/sample-catalog-private:latest -u <username> -p <password>
```

**Note**: This serves as a convenient way to connect to private 
registries but is insecure locally as your credentials are saved in
your shell's history. If you would like to remain completely secure then
login via `<container_runtime> login` and pull the image locally. dcp 
will then be able to notice the image locally pulled and process it.

## FAQ

**Q**: I hit an unexpected error unpacking the root filesystem of an image: `trying to unpack outside of destination path`. How can I avoid this?

**A**: dcp relies on the underlying `tar` Rust library to unpack the image filesystem represented as a tar file. The [unpack](https://docs.rs/tar/latest/tar/struct.Archive.html#method.unpack) command is senstive in that it will not write files outside of the path specified by the destination. So things like symlinks will cause errors when unpacking. Whenever possible, use the `-c` flag to specify a directory to unpack, instead of the filesystem root, to avoid this error. 

------------------
**Q**: I would like to use dcp to pull content from an image but I don't know where in the image the content is stored. Is there an `ls` command or similar functionality in dcp? 

**A**: Checkout the excellent [dive tool](https://github.com/wagoodman/dive) to easily explore a container filesystem by layer. After finding the path of the files to copy, you can then use dcp to extract just those specific files. 

------------------
**Q**: Is dcp supported on Windows?

**A**: Windows support is being tracked in [#14](https://github.com/exdx/dcp/issues/14). We are looking for someone with access to a Windows machine to help work on Windows support. Outside of changing the socket paths, there may not be any other work required to support Windows.


## Testing

If you would like to run the test suite, you just need to run the standard cargo command. This will run all relevant
unit, integration and documentation tests.

```
$ cargo test
```
