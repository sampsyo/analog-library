Analog Library
==============

It's just a super simple way to render metadata about a paper DOI.
The idea is to provide a good URL to send to a friend when you're talking about a paper.
Analog Library pages are crap-free and load quickly.

You can see a running instance [at `al.radbox.org`][al].

To run it yourself, you can just `cargo build --release` or [download a pre-built binary][releases].
Run the executable to launch the server on port 8118.
Analog Library will create a cache database in the working directory.
Set the `MAILTO` environment variable to identify yourself in your requests to the [Crossref API][crapi], which is something they appreciate.

Analog Library is by [Adrian Sampson][adrian].
The license is [MIT][].

[al]: https://al.radbox.org
[crapi]: https://www.crossref.org/documentation/retrieve-metadata/rest-api/
[adrian]: https://www.cs.cornell.edu/~asampson/
[mit]: https://opensource.org/license/mit
[releases]: https://github.com/sampsyo/analog-library/releases
