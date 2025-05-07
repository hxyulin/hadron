# Kernel Config (kconfig)

This crate provides the implementation for the KConfig.toml parser.
This is heavily inspired by kconfig from the Linux kernel.

## Design

The KConfig.toml file is a simple TOML file that contains a list of configuration options.
Configuration options are scanned by walking the directory tree and looking for files that match the name `kconfig.toml`.

By default, the kconfig found in a nested directory will be prefixed with the name of the directory. This can be overriden using the `prefix` field.

