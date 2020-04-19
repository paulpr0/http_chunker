# http_chunker

http_chunker is a simple utility for encoding data in the format expeted when 

Transfer-Encoding:chunked

is set on an http response.

http_chunker has two options - a desired chunk size, and a max wait time. It will try to read chunk size bytes from input and write them to the output, but if the max wait is reached, it will encode and write out any data it has read. 

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
![Rust](https://github.com/paulpr0/http_chunker/workflows/Rust/badge.svg)

## Examples

```bash
$ http_chunker --help
Usage: http_chunker [-w <max-wait>] [-s <chunk-size>]

Read from stdin, chunk and write back out again. After a set delay, write out anything which is queued

Options:
  -w, --max-wait    maximum time (milliseconds) to wait before writing out a
                    partially filled buffer. Set to 0 for no waiting, and -1 to
                    wait forever
  -s, --chunk-size  chunk size
  --help            display usage information
```

```bash
$ echo "hello world!" | http_chunker

13
hello world!
```

```bash
$ echo "hello world!" | ./part94 -s 4

4
hell
4
o wo
4
rld!
1

```
