# Contributing

If you want to contribute to cynic, that's great!  There's plenty of
improvements to make, features that still need built and probably a bug or two
hiding in there.

Please note we have a [code of conduct][COC], please follow it in all your
interactions with the project.

There's a few different ways to contribute:

- If you've found a bug or have an idea for a feature you'd like to implement,
  please [create an issue][NewIssue] to discuss or send a message [on
  discord][Discord].
- If you'd like to contribute to an existing issue feel free to comment on the
  issue and let us know.  If anything isn't clear someone will be happy to
  explain (the project is still fairly new, and I have treated the issue
  tracker like a notepad occasionally, sorry about that 😬).
- If you'd like to contribute but you're not sure how:
  - We have [Good First Issues][GFI] labelled.
  - We [use milestones][Milestones] to plan what's going into the next release,
    any free work in there would be great to pick up.
  - The documentation (both on [docs.rs](https://docs.rs/cynic) & on
    [cynic-rs.dev](https://cynic-rs.dev)) could always be improved - try
    following one of the guides, and let us know what wasn't clear, what you
    thinks missing or even what
  - You can just try to use cynic in a project - we'd appreciate any bug
    reports or feedback on the API, and it's always nice seeing what people
    build.

## Getting Help

If you have any questions about how to do anything or otherwise need help, please just ask.  
You can do this by:

1. [Joining the discord server][Discord]
2. [Asking in Discussions][Discussions]

## Coding Guidelines

1. `cargo fmt` everything
2. Don't use unsafe
3. Don't leave warnings in the codebase.

## Pull Request Process

1. Ensure the build passes and your code meets the above guidelines.
2. Ideally make sure you have some tests for any new functionality you've added.
3. Update CHANGELOG.md with a new entry for your changes (under unreleased)
4. Please write a reasonably detailed pull request message to help reviewer understand the
   context around & reason for the changes.
5. A maintainer will review the PR as soon as possible, and once it is approved will merge
   and make a release.
   
[COC]: ./CODE_OF_CONDUCT.md
[Discord]: https://discord.gg/Y5xDmDP
[Discussions]: https://github.com/obmarg/cynic/discussions/new
[GFI]: https://github.com/obmarg/cynic/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[Milestones]: https://github.com/obmarg/cynic/milestones
