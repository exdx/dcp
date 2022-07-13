# dcp: docker cp made easy

## Summary

Containers are great tools that can encapsulate an application and its dependencies,
allowing apps to run anywhere in a streamlined way. Some container images contain
commands to start a long-lived binary, whereas others may simply contain data
that needs to be available in the environment (for example, a Kubernetes cluster).
For example, the operator-framework `bundle` format is an example of using
containers image to store manifests, which can be unpacked on-cluster and made
available to end users.

One of the downsides of using container images to store data is that they are
necessarily opaque. There's no way to quickly tell what's inside the image, although
the hash digest is useful in seeing whether the image has changed from a previous
version. The options are to use `docker cp` or something similar using podman
or containerd.

Using `docker cp` by itself can be cumbersome. Say you have a remote image
somewhere in a registry. You have to pull the image, create a container from that
image, and only then run `docker cp <container-id>` using an unintuitive syntax for selecting
what should be copied to the local filesystem.

`dcp` is a simple binary that attempts to simplify this workflow. A user can simply
say `dcp <image-name>` and it can extract the contents of that image on to the
local filesystem. It can also simply print the contents of the image to stdout, and
not create any local files.

## Implementation

Because there wasn't a suitable `containerd` client implementation in Rust at the time
of writing, `dcp` relies on the docker APIs provided by an external crate. Unfortunately,
this limits `dcp` to only working on systems where docker is the container runtime.

## Flags and Examples

As an example, lets try
`dcp tyslaton/sample-catalog:v0.0.4 -d output -p configs`

This command pulls down the request image, only extracting
the `configs` directory (via the `-p` flag) and copying it to the `output` directory
locally (via the `-d` flag). 
