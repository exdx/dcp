# How to contribute

I'm really glad you're reading this, thank you for your interest in contributing! 🙂

The best way to communicate in this project is via GitHub issues. Issues are triaged frequently. 

## Submitting changes

The simplest way to submit a change is via a PR. For larger features, opening an issue first and discussing the feature would be preferred. 

Always write a clear log message for your commits. One-line messages are fine for small changes, but bigger changes should look like this:

    $ git commit -m "A brief summary of the commit
    > 
    > A paragraph describing what changed and its impact."

## Coding conventions

Running clippy or `cargo fmt` before pushing code is encouraged, since there is a lint GitHub Action that will fail if code is not formatted correctly.

Beyond that, it's encouraged to write concise code: if something can be expressed in one line versus five, the one liner is preferred. 

## Testing

Testing is pretty straightforward. We have a series of e2e tests that run against both docker and podman runtimes. Running `cargo test` locally would enable the
test suite to run against either of the runtimes available locally. 

Thank you for contributing! 


