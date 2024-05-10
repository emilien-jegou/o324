# o324

An open source CLI and GUI time tracker for linux developed in rust.

> ⚠️ Project in active development.

# Introduction

This project aims to create an open-source time tracking solution featuring a customizable storage layer. Many us value having control over their data, yet often find themselves tied to proprietary options due to a lack of suitable open-source alternatives. This project seeks to address this gap by offering a software solution that operates effectively without compromising on data control.

# Roadmap

## Storage types
- [~] git
- [ ] file storage (no sync)
- [ ] server
- [ ] P2P

## CLI commands
- [x] cancel
- [x] delete
- [x] edit
- [x] init
- [x] log
- [x] restart
- [x] start
- [ ] stats
- [ ] status
- [x] stop
- [x] sync

## GUI
- [~] basic actions
  - [x] create
  - [~] edit
  - [x] delete
  - [x] stop
  - [x] task list
  - [x] synchronization
  - [ ] settings
- [x] hot-reload on change (IPC)
- [ ] design implementation
  - [ ] onboarding
  - [~] latest task view
  - [~] calendar view
  - [~] settings view
- [ ] auto synchronization
- [ ] CLI interface

## Core
- [x] configuration profiles
- [~] documentation website

## Mobile

Currently experimenting with [Tauri mobile](https://beta.tauri.app/fr/blog/tauri-mobile-alpha/#preview). Check the `dev-android` branch.

# Screens

![Design file](https://raw.githubusercontent.com/emilien-jegou/o324/main/docs/screen_1.png "Design file")
