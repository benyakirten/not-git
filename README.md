# What am I looking at?

Based off of the [CodeCrafters Build your own Git](https://app.codecrafters.io/courses/git/overview) challenge, this is an implementation of a few of Git's more basic properties. I did this because I typically have worked mostly on web-based projects, and I wanted to try something different. Not only that, but I got some practice with Rust and have got much more experience with bitwise operators (which almost never come up in JavaScript-related technologies). I made the following additions and changes:

1. Add CI/tests of my own (TODO).
1. Add documentation (TODO).
1. Change the app from CLI-based to a library (TODO).
1. Add refs, including the `update-refs` plumbing command, and the `branch`, `commit` and `checkout` porcelain commands.

I have no current plans to complete functionality much beyond what is currently available. Unfortunately, I have to choose where I invest my time. While I would enjoy finishing every command and adding some additions of my own, there's no way I would have the time to do so.

# Can I use the library?

I doubt that the library will be of any use to you. That said, you can learn some of the basics of how git works in terms of encoding, how files are stored and how packfiles work.

# Future Additions

Beside the additions marked as TODO in the top section:

1. Fix branch discovery, especially remote branches that may not have been downloaded.
