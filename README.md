# md2html
Simple markdown to html parser for my own blog.

## Note
This code does not aim to be a compliant markdown parser by any means, if you are looking for compliant parsers, see:
- [pulldown-mark](https://lib.rs/crates/pulldown-cmark)
- [comrak](https://lib.rs/crates/comrak)

### Why not use an existing library?
I didn't want to fight a library to do something I could do myself. In case I wanted to implement a custom behavior, I preferred to not spend time searching on how can it be done with 3rd people software, and just do it on my own.

### Why not using `Regex`s?
You also may ask yourself, why not use just `Regex`s, like they do [here](https://betterprogramming.pub/create-your-own-markdown-parser-bffb392a06db)? Mainly, because I wanted to program in Rust, I know I could have developed it mostly in `Regex`, but I wanted to program the logic in Rust, not finding a complex `Regex`s to fit any custom need I may have.

I wanted to try to write it without looking anywhere, without any tutorial, what could I write out of my imagination.

### Why not use a more state-of-the-art approach, like ANTLR, or using state-machines?
Simplicity, I didn't want to build a resilient, spec-compliant parser, just a quick binary to parse the subset of markdown I'm going to use in my blogs.