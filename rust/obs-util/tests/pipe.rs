//! Tier 1: safe-core tests for `libobs/util/pipe.c` and the POSIX half of
//! `libobs/util/pipe-posix.c`.
//!
//! Each test names the cmocka case in `test/cmocka/test_pipe.c` it mirrors.
//! Raw-pointer C-ABI properties (the argv NULL terminator, null-handle
//! calls) live in `tests/pipe_parity.rs`, like the file-serializer port:
//! Tier 1 only touches the safe API. `os_process_args_destroy(NULL)` has no
//! assertion and nothing to mirror: dropping `Args` is implicit.

use obs_c_oracle as _;
use obs_util::pipe::Args;
#[cfg(unix)]
use obs_util::pipe::Pipe;

/// Mirrors `test_args_create` in `test/cmocka/test_pipe.c`.
#[test]
fn args_create_mirrors_test_args_create() {
    let args = Args::new(b"prog");
    assert_eq!(args.argc(), 1);
    assert_eq!(args.arg(0), Some(b"prog".as_slice()));
    assert_eq!(args.arg(1), None);
}

/// Mirrors `test_args_add_arg_order`.
#[test]
fn args_add_arg_order_mirrors_test_args_add_arg_order() {
    let mut args = Args::new(b"prog");
    args.add_arg(b"one");
    args.add_arg(b"two");
    args.add_arg(b"three");

    assert_eq!(args.argc(), 4);
    assert_eq!(args.arg(0), Some(b"prog".as_slice()));
    assert_eq!(args.arg(1), Some(b"one".as_slice()));
    assert_eq!(args.arg(2), Some(b"two".as_slice()));
    assert_eq!(args.arg(3), Some(b"three".as_slice()));
    assert_eq!(args.arg(4), None);
}

/// Mirrors `test_args_add_argf`: the core stores the already-formatted
/// bytes (`"42-abc"`); `vsnprintf` itself stays in `util/pipe-variadic.c`.
#[test]
fn args_add_formatted_mirrors_test_args_add_argf() {
    let mut args = Args::new(b"prog");
    args.add_formatted(b"42-abc");
    args.add_arg(b"after");

    assert_eq!(args.argc(), 3);
    assert_eq!(args.arg(1), Some(b"42-abc".as_slice()));
    assert_eq!(args.arg(2), Some(b"after".as_slice()));
    assert_eq!(args.arg(3), None);
}

/// Mirrors `test_args_verbatim`: spacing, quotes and empty args pass through.
#[test]
fn args_verbatim_mirrors_test_args_verbatim() {
    let mut args = Args::new(b"prog");
    args.add_arg(b"has space");
    args.add_arg(b"say \"hi\" 'there'");
    args.add_arg(b"");

    assert_eq!(args.argc(), 4);
    assert_eq!(args.arg(1), Some(b"has space".as_slice()));
    assert_eq!(args.arg(2), Some(b"say \"hi\" 'there'".as_slice()));
    assert_eq!(args.arg(3), Some(b"".as_slice()));
    assert_eq!(args.arg(4), None);
}

/// `bstrdup` copies with `strlen` semantics, so an embedded NUL truncates.
#[test]
fn args_truncates_at_nul_like_bstrdup() {
    let mut args = Args::new(b"pro\0g");
    args.add_arg(b"a\0b");

    assert_eq!(args.argc(), 2);
    assert_eq!(args.arg(0), Some(b"pro".as_slice()));
    assert_eq!(args.arg(1), Some(b"a".as_slice()));
}

#[cfg(unix)]
mod process {
    use super::*;

    /// Mirrors the `read_all` helper in `test/cmocka/test_pipe.c`: read
    /// until EOF (read returns 0); returns total bytes.
    fn read_all(pipe: &mut Pipe, mut reader: impl FnMut(&mut Pipe, &mut [u8]) -> usize) -> Vec<u8> {
        let mut out = Vec::new();
        let mut chunk = [0u8; 64];
        loop {
            let n = reader(pipe, &mut chunk);
            if n == 0 {
                break;
            }
            out.extend_from_slice(&chunk[..n]);
        }
        out
    }

    /// Mirrors `test_pipe_create_read`.
    #[test]
    fn pipe_create_read_mirrors_test_pipe_create_read() {
        let mut pipe = Pipe::create(Some(b"printf 'hello'"), b"r").expect("spawn sh");
        let out = read_all(&mut pipe, |p, buf| p.read(buf));
        assert_eq!(out, b"hello");
        assert_eq!(pipe.destroy(), 0);
    }

    /// Mirrors `test_pipe_exit_code`.
    #[test]
    fn pipe_exit_code_mirrors_test_pipe_exit_code() {
        let mut pipe = Pipe::create(Some(b"exit 3"), b"r").expect("spawn sh");
        let out = read_all(&mut pipe, |p, buf| p.read(buf));
        assert!(out.is_empty());
        assert_eq!(pipe.destroy(), 3);
    }

    /// Mirrors `test_pipe_exit_code_signed_char`: the status is cast through
    /// the platform `char`, so 255 becomes -1 where `char` is signed.
    #[test]
    fn pipe_exit_code_signed_char_mirrors_test_pipe_exit_code_signed_char() {
        let mut pipe = Pipe::create(Some(b"exit 255"), b"r").expect("spawn sh");
        let out = read_all(&mut pipe, |p, buf| p.read(buf));
        assert!(out.is_empty());
        assert_eq!(
            pipe.destroy(),
            255u8 as core::ffi::c_char as core::ffi::c_int
        );
    }

    /// Mirrors `test_pipe_create2_stdout_stderr`.
    #[test]
    fn pipe_create2_stdout_stderr_mirrors_test_pipe_create2_stdout_stderr() {
        let mut args = Args::new(b"/bin/sh");
        args.add_arg(b"-c");
        args.add_arg(b"printf abc; printf err 1>&2");

        let mut pipe = Pipe::create2(Some(&args), b"r").expect("spawn sh");
        let out = read_all(&mut pipe, |p, buf| p.read(buf));
        assert_eq!(out, b"abc");
        let err = read_all(&mut pipe, |p, buf| p.read_err(buf));
        assert_eq!(err, b"err");
        assert_eq!(pipe.destroy(), 0);
    }

    /// Mirrors `test_pipe_create2_missing_binary`: a bad path is `None`.
    #[test]
    fn pipe_create2_missing_binary_mirrors_test_pipe_create2_missing_binary() {
        let args = Args::new(b"/nonexistent/obs-test-binary");
        assert!(Pipe::create2(Some(&args), b"r").is_none());
    }

    /// Mirrors `test_pipe_create2` with null args: C dereferences the null;
    /// the core returns `None` instead (listed in `pipe_parity.rs`).
    #[test]
    fn pipe_create2_null_args_is_none() {
        assert!(Pipe::create2(None, b"r").is_none());
    }

    /// Mirrors `test_pipe_write_mode`: full write, reads yield nothing.
    #[test]
    fn pipe_write_mode_mirrors_test_pipe_write_mode() {
        let mut pipe = Pipe::create(Some(b"cat > /dev/null"), b"w").expect("spawn sh");
        assert_eq!(pipe.write(b"some data"), b"some data".len());
        assert_eq!(pipe.read(&mut [0u8; 4]), 0);
        assert_eq!(pipe.destroy(), 0);
    }

    /// Mirrors `test_pipe_write_on_read_pipe`.
    #[test]
    fn pipe_write_on_read_pipe_mirrors_test_pipe_write_on_read_pipe() {
        let mut pipe = Pipe::create(Some(b"exit 0"), b"r").expect("spawn sh");
        assert_eq!(pipe.write(b"x"), 0);
        assert_eq!(pipe.destroy(), 0);
    }

    /// Mirrors `test_pipe_type_handling`: only a leading `r` reads.
    #[test]
    fn pipe_type_handling_mirrors_test_pipe_type_handling() {
        let mut pipe = Pipe::create(Some(b"cat > /dev/null"), b"x").expect("spawn sh");
        assert_eq!(pipe.write(b"ab"), 2);
        assert_eq!(pipe.destroy(), 0);
    }

    /// Mirrors `test_pipe_null_arguments`: a null command is `None`.
    #[test]
    fn pipe_null_command_is_none_mirrors_test_pipe_null_arguments() {
        assert!(Pipe::create(None, b"r").is_none());
    }

    /// A signaled child destroys to its pid, mirroring the `waitpid`
    /// fallthrough in `os_process_pipe_destroy`.
    #[test]
    fn pipe_signaled_child_destroys_to_pid() {
        let pipe = Pipe::create(Some(b"sleep 30"), b"r").expect("spawn sh");
        let pid = pipe.pid();
        // SAFETY: `pid` is our own child, signaled exactly once.
        unsafe {
            assert_eq!(libc::kill(pid as libc::pid_t, libc::SIGKILL), 0);
        }
        assert_eq!(pipe.destroy(), pid as core::ffi::c_int);
    }
}
