# emailer

## What's the same

As mentioned above, this project largely follows the guiding hand of zero2prod's author.

## What's different

The Dockerfile is different. Check it out.

- use cargo-chef to cache dependencies without invalidating them on each build.

Since I use an M1 mac for development, I implemented the following tricks to save compilation time.

- using sscache
- changing linker to zld
