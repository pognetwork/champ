# Continuous Delivery and Operations @ pog.network

![](cicd.drawio.svg)

## Automated Testing

All commits are automatically build & tests are run. Further, we use `cargo check` and `clippy` for static code analysis.

## Automated Nightly Builds

All commits to our main `master` branch are automatically build for `linux_x86` and made availabe as nightly builds using github artifacts.

## Automated Releases

We have a custom workflow for automatically creating releases and release notes. By using [conventional-commits](https://www.conventionalcommits.org/en/v1.0.0/) as our convention for comit messages in `main`, we can automatically calculate the next version bump based on semantic versioning and generate a meaningful changelog. To make creating releases as easy and fast as possible, they can either be created by marking a pull request with a `release` label or by including `[new_release]` in the commit message. This triggeres our automated release process which updates the version number in all packages in this monorepo and commits the changes to `main`.

After a new release has been finally released, a new build job starts for every architecture on native windows, linux and macos vms.

## Documentation

We manage documentation in our `pog.network` repo, which combines the markdown files in all of our projects managed through git submodules. To keep the submodules up to date, we have a special ci workflow which only runs on changes to markdown files and creates a new commit in the `pog.network` repo updating the relevant submodule. Here, every new commit is build and published to our main website at [https://pog.network](https://pog.network) using cloudflare pages with our customized mkdocs theme.
