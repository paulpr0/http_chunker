use argh::FromArgs;
use tokio::{prelude::*,runtime,io,time::timeout_at};
use std::time::Duration;
#[derive(FromArgs)]
/// Read from stdin, chunk and write back out again.
/// After a set delay, write out anything which is queued
struct Opts {
    /// maximum time (milliseconds) to wait before writing out a partially filled buffer.
    /// Set to 0 for no waiting, and -1 to wait forever
    #[argh(option, short='w', default = "2000")]
    max_wait: i64,

    /// chunk size
    #[argh(option, short='s', default = "1024")]
    chunk_size: usize,
}

//need to make the timeout take into acct elapsed time...

async fn write_chunk<F: io::AsyncWrite + Unpin>(out: &mut F, buf:&[u8]) {
    out.write(format!("\r\n{}\r\n", buf.len()).as_bytes()).await.expect("Output error");
    out.write(buf).await.expect("Output error");
}

async fn do_stuff<I: io::AsyncRead + Unpin, O: io::AsyncWrite + Unpin>(
    opts:Opts, i: &mut I, o:&mut O) -> () {
    let mut buf = vec![0;opts.chunk_size];
    let mut bytes_read=0;
    let mut timeout_time = tokio::time::Instant::now() + Duration::from_millis(opts.max_wait as u64);
    loop {
        let res = if opts.max_wait>0 {
            match timeout_at(timeout_time, i.read(&mut buf[bytes_read..])).await {
             //   match timeout(tokio::time::Duration::from_millis(opts.max_wait as u64), i.read(&mut buf[bytes_read..])).await {
                Ok(Ok(n)) if n == 0 => break,
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    println!("input error: {}", e);
                    return;
                }
                Err(_) => 0
            }
        } else {
            match i.read(&mut buf[bytes_read..]).await {
                Ok(n) if n==0 => break,
                Ok(n) => n,
                Err(e) => {
                    println!("input error: {}", e);
                    return;
                }
            }
        };
        bytes_read+= res;
        if bytes_read == opts.chunk_size || (bytes_read>0 && res==0) {
            write_chunk(o, &buf[0..bytes_read]).await;
            bytes_read=0;
            timeout_time = tokio::time::Instant::now() + Duration::from_millis(opts.max_wait as u64);
        }
        else if bytes_read == 0 {
            timeout_time = tokio::time::Instant::now() + Duration::from_millis(opts.max_wait as u64);
        }

    }
    if bytes_read>0 {
        write_chunk(o, &buf[0..bytes_read]).await;
    }
}

fn main() -> () {
    let opts:Opts = argh::from_env();
    let mut rt = runtime::Runtime::new().unwrap();
    rt.block_on(do_stuff(opts, &mut io::stdin(), &mut io::stdout()));
}

#[cfg(test)]
mod test {
    use tokio_test::io::Builder;
   // use tokio::io;
    use crate::{Opts, do_stuff};
    use std::time::Duration;

    #[test]
    fn split_read() {
       // let error = io::Error::new(io::ErrorKind::Other, "test");
        let mut min = Builder::new()
            .read(b"hello ")
          //  .read_error(error)
            .read(b"world!")
            .build();
        let mut mout = Builder::new()
            .write(b"\r\n12\r\nhello world!")
            .build();
        let opts = Opts {max_wait: 10, chunk_size:50};

        tokio_test::block_on(
            do_stuff(opts, &mut min,&mut mout));

    }

    #[test]
    fn write_called_once_buf_full() {
        let mut min = Builder::new()
            .read(b"A test ")
            //  .read_error(error)
            .read(b"world!")
            .build();
        let mut mout = Builder::new()
            .write(b"\r\n5\r\nA tes")
            .write(b"\r\n5\r\nt wor")
            .write(b"\r\n3\r\nld!")
            .build();
        let opts = Opts {max_wait: 10, chunk_size:5};

        tokio_test::block_on(
            do_stuff(opts, &mut min,&mut mout));
    }

    #[test]
    fn slow_output_does_not_drop_data() {
        let mut min = Builder::new()
            .read(b"A test ")
            //  .read_error(error)
            .read(b"world!")
            .build();
        let mut mout = Builder::new()
            .write(b"\r\n5\r\nA tes")
            .wait(Duration::from_millis(100))
            .write(b"\r\n5\r\nt wor")
            .wait(Duration::from_millis(100))
            .write(b"\r\n3\r\nld!")
            .build();
        let opts = Opts {max_wait: 10, chunk_size:5};

        tokio_test::block_on(
            do_stuff(opts, &mut min,&mut mout));

    }

}
