# o324

An open source CLI and GUI time tracker for linux developed in rust.

> ⚠️ Project in active development.

# Introduction

This project aims to create a cross-platform open-source time tracking solution featuring a customizable storage layer. Many of us value having control over their data, yet often find themselves tied to proprietary options due to a lack of suitable open-source alternatives. This project seeks to address this gap by offering a software solution that operates effectively without compromising on data control.

## Git as a database?

The first database layer that was developed for the tool is Git, but why would you use Git as a storage layer? here are my argument for it:
- git is widespread, you can choose to host your backend anywhre, most of the time for free (gitlab, bitbucket, github) or self-host it.
- git already offer capabilities for reconciliating divergent changes, making a CRDT (conflict-free replicated data type) implementation on top of it trivial.
- git use the algorithm that most blockchain use for block validation (in this case commit validation), ensuring data integrity accross different devices.
- It goes well with my idea of having human readable storage data for analytics (json, yaml and toml are supported).
- All changes are reversible

There are still some drawbacks of using git as a storage layer, the main one is that it may be too technical for your general user, which is why the storage layer and configuration was developed in a way where future storage layer can be implemented (such as P2P or server-based).

# Roadmap

For more information about the roadmap check the [roadmap.md](https://github.com/emilien-jegou/o324/blob/main/roadmap.md) file in this repository.

# Screens

![Design file](https://raw.githubusercontent.com/emilien-jegou/o324/main/docs/screen_1.png "Design file")
