# Continuous Delivery and Operations @ pog.network

[![](cicd.drawio.svg)](cicd.drawio.svg)

## Automated Testing

All commits are automatically built & tests are run. Further, we use `cargo check` and `clippy` for static code analysis.

## Automated Nightly Builds

All commits to our main `master` branch are automatically built for `linux_x86` and made available as nightly builds using GitHub artifacts.

## Automated Releases

We have a custom workflow for automatically creating releases and release notes. We can automatically calculate the next by using [conventional-commits](https://www.conventionalcommits.org/en/v1.0.0/) as our convention for commit messages in `main` version bump based on semantic versioning and generate a meaningful changelog. Creating releases as easy and fast as possible can either be made by marking a pull request with a `release` label or by including `[new_release]` in the commit message. This triggers our automated release process, which updates the version number in all packages in this monorepo and commits the changes to `main`.

After a new release has been finally released, a new build job starts for every architecture on native Windows, Linux, and macOS VMs.

## Documentation

We manage documentation in our `pog.network` repo, which combines the markdown files in our projects managed through git submodules. To keep the submodules up to date, we have a unique ci workflow that only runs on changes to markdown files and creates a new commit in the `pog.network` repo updating the relevant submodule. Every new commit is built and published to our main website at [https://pog.network](https://pog.network) using Cloudflare pages with our customized mkdocs theme.
