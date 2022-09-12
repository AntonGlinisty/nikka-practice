#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    error::Error,
    ffi::CString,
    io,
    io::prelude::*,
    str,
};

use nix::{
    fcntl::{
        OFlag,
        open,
    },
    libc::{
        STDIN_FILENO,
        STDOUT_FILENO,
    },
    sys::{
        stat::Mode,
        wait::waitpid,
    },
    unistd::{
        ForkResult,
        close,
        dup2,
        execvp,
        fork,
        pipe,
        read,
        write,
    },
};

// TODO: your code here.

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: your code here.
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use std::process::exit;

    use nix::{
        errno::Errno,
        libc::EINTR,
        sys::wait::WaitStatus,
    };

    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize(""), Vec::<&str>::new());
        assert_eq!(tokenize("foo bar zog"), vec!("foo", "bar", "zog"));
        assert_eq!(
            tokenize(">foo bar <  zog"),
            vec!(">", "foo", "bar", "<", "zog"),
        );
        assert_eq!(
            tokenize(">foo bar|zog <xxx"),
            vec!(">", "foo", "bar", "|", "zog", "<", "xxx"),
        );
    }

    #[test]
    fn test_next() {
        let tokens = tokenize(">foo bar < zog | wc -l");
        let mut start = 0;

        let (infile, outfile, cmd) = get_command(&tokens, &mut start);
        assert_eq!(Some("zog"), infile);
        assert_eq!(Some("foo"), outfile);
        assert_eq!(vec!("bar"), cmd);

        let (infile, outfile, cmd) = get_command(&tokens, &mut start);
        assert_eq!(None, infile);
        assert_eq!(None, outfile);
        assert_eq!(vec!("wc", "-l"), cmd);
        assert_eq!(start, 8);
    }

    #[test]
    fn test_shell() {
        let (inpipe, outchild) = pipe().unwrap();
        let (inchild, outpipe) = pipe().unwrap();

        let pid = match unsafe { fork().unwrap() } {
            ForkResult::Parent { child } => child,
            ForkResult::Child => {
                close(inpipe).unwrap();
                close(outpipe).unwrap();

                dup2(outchild, STDOUT_FILENO).unwrap();
                close(outchild).unwrap();

                dup2(inchild, STDIN_FILENO).unwrap();
                close(inchild).unwrap();

                main().unwrap();
                exit(0);
            },
        };

        let send = |s: &str| {
            assert_eq!(write(outpipe, s.as_bytes()).unwrap(), s.len());

            let str = "Hello";

            let mut i = 0;
            while i != str.len() {
                match write(outpipe, str[i ..].as_bytes()) {
                    Ok(size) => i += size,
                    Err(Errno::EINTR) => continue,
                    err => {
                        err.unwrap();
                    },
                }
            }
        };

        let recv = || -> Vec<u8> {
            let mut buf = Vec::new();
            buf.resize(1024, 0);

            let n = read(inpipe, buf.as_mut_slice()).unwrap();

            buf.resize(n, 0);
            buf
        };

        send("echo hello\n");
        assert_eq!(str::from_utf8(recv().as_slice()).unwrap(), "hello\n");

        close(outpipe).unwrap();
        assert_eq!(waitpid(pid, None).unwrap(), WaitStatus::Exited(pid, 0));
    }
}
