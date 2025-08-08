# iOS Platform Support Rules

## Three Key Rules

1. Check the `plan/ios_support.md` file after every commit/major feature and update it as needed

2. DO NOT create ANY other markdown files besides specified ones. Do not create ANY bash script or example rust or markdown file if it is not in rust's native `examples` folder. DO NOT CREATE BASH SETUP SCRIPTS OR MIGRATION SCRIPTS.

3. Do not create "legacy" or `_old` files. Any change should fully be implemented and bring the rest of the framework up to date with it. We _never_ want to interoperate with old code. Bump the version if needed for semver.
