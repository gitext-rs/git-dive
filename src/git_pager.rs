const DEFAULT_ENV: &[(&str, &str)] = &[("LESS", "FRX"), ("LV", "-c")];
const REQUIRED_ENV: &[(&str, &str)] = &[("LESSCHARSET", "UTF-8")];

pub(crate) struct Pager {
    cmd: Option<std::process::Command>,
}

impl Pager {
    pub(crate) fn stdout(args: &str) -> Self {
        let cmd = anstream::stdout()
            .is_terminal()
            .then(|| parse(args))
            .flatten();
        Self { cmd }
    }

    pub(crate) fn start(&mut self) -> ActivePager {
        let stdout = anstream::stdout().lock();
        if let Some(cmd) = &mut self.cmd {
            // should use pager instead of stderr
            if let Ok(p) = cmd.spawn() {
                let stderr = anstream::stderr()
                    .is_terminal()
                    .then(|| anstream::stderr().lock());
                ActivePager {
                    primary: stdout,
                    _secondary: stderr,
                    pager: Some(p),
                }
            } else {
                ActivePager {
                    primary: stdout,
                    _secondary: None,
                    pager: None,
                }
            }
        } else {
            ActivePager {
                primary: stdout,
                _secondary: None,
                pager: None,
            }
        }
    }
}

pub(crate) struct ActivePager {
    primary: anstream::AutoStream<std::io::StdoutLock<'static>>,
    _secondary: Option<anstream::AutoStream<std::io::StderrLock<'static>>>,
    pager: Option<std::process::Child>,
}

impl ActivePager {
    pub(crate) fn as_writer(&mut self) -> std::io::Result<&mut dyn std::io::Write> {
        if let Some(pager) = &mut self.pager {
            pager
                .stdin
                .as_mut()
                .map(|s| {
                    let s: &mut dyn std::io::Write = s;
                    s
                })
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "could not access pager stdin",
                    )
                })
        } else {
            Ok(&mut self.primary)
        }
    }
}

impl Drop for ActivePager {
    fn drop(&mut self) {
        if let Some(pager) = &mut self.pager {
            let _ = pager.wait();
        }
    }
}

fn parse(args: &str) -> Option<std::process::Command> {
    let mut args = shlex::Shlex::new(args);
    let cmd = args.next()?;
    if cmd == "cat" {
        return None;
    }
    let mut cmd = std::process::Command::new(cmd);
    cmd.stdin(std::process::Stdio::piped());
    cmd.args(args);
    cmd.envs(REQUIRED_ENV.iter().copied());
    cmd.envs(
        DEFAULT_ENV
            .iter()
            .copied()
            .filter(|(k, _)| std::env::var_os(k).is_none()),
    );
    Some(cmd)
}
