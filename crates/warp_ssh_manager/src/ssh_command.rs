//! 把 `SshServerInfo` 拼成 `ssh ...` 命令,并派生测试连接的子进程。
//!
//! 写入 PTY 时调 `build_ssh_command_line`,会用 shell-escape 引用每个 arg,
//! 防止用户名 / host / key_path 里的空格或单引号破坏命令行。
//!
//! ## 密码认证安全 & 跨平台兼容性
//!
//! **非 Windows**:`ssh` 在 pipe stdin 模式下能正常从 stdin 读密码,采用一次性
//! stdin 注入(`build_password_auth_stdin`)。密码全程只在内存中以
//! `Zeroizing<String>` 形式持有,不进 argv,不会出现在 `/proc/<pid>/cmdline`、
//! `ps` 等同机可读的进程信息里(对 sshpass `-p` 模式的修复)。
//!
//! **Windows**:Win32-OpenSSH 即便 stdin 是 pipe,也会因为
//! `CREATE_NO_WINDOW`(无控制台)拒绝从 stdin 读密码,打印
//! `GetConsoleMode on STD_INPUT_HANDLE failed` 后挂死,见
//! PowerShell/Win32-OpenSSH issue #1470。绕开方案是 `SSH_ASKPASS`:
//! 写一个临时 .cmd 脚本,ssh 派生它并把 stdout 当密码读,完全绕过 stdin
//! 和控制台。`SSH_ASKPASS_REQUIRE=force` 强制走 askpass 路径。密码本身
//! 通过临时文件传给 askpass 脚本(不写 env var,降低泄漏面),整个生命周期
//! 由 `AskpassSession` RAII 守卫保证 ssh 退出后立即清理。

use crate::types::{AuthType, ConnectionStatus, SshServerInfo};
#[cfg(not(windows))]
use futures_lite::io::AsyncWriteExt as _;
use std::borrow::Cow;
use std::process::Stdio;
use std::time::Duration;
use zeroize::Zeroizing;

pub fn build_ssh_args(server: &SshServerInfo) -> Vec<String> {
    let mut args: Vec<String> = vec!["ssh".into()];
    if server.port != 22 {
        args.push("-p".into());
        args.push(server.port.to_string());
    }
    if matches!(server.auth_type, AuthType::Key | AuthType::OneKey) {
        if let Some(path) = server.key_path.as_deref() {
            if !path.is_empty() {
                args.push("-i".into());
                args.push(path.to_string());
            }
        }
    }
    let target = if server.username.is_empty() {
        server.host.clone()
    } else {
        format!("{}@{}", server.username, server.host)
    };
    args.push(target);
    args
}

pub fn build_ssh_command_line(server: &SshServerInfo) -> String {
    let args = build_ssh_args(server);
    args.iter()
        .map(|a| shell_escape::unix::escape(Cow::Borrowed(a.as_str())).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

const TEST_TIMEOUT: Duration = Duration::from_secs(10);

pub struct ConnectionTestResult {
    pub status: ConnectionStatus,
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
}

pub async fn test_connection(
    server: &SshServerInfo,
    password: Option<Zeroizing<String>>,
) -> ConnectionTestResult {
    let start = instant::Instant::now();

    let result = match server.auth_type {
        AuthType::Key => test_key_auth(server).await,
        AuthType::Password | AuthType::OneKey => test_password_auth(server, password).await,
    };

    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(()) => ConnectionTestResult {
            status: ConnectionStatus::Online,
            latency_ms: Some(latency),
            error_message: None,
        },
        Err(e) => ConnectionTestResult {
            status: ConnectionStatus::Offline,
            latency_ms: Some(latency),
            error_message: Some(e),
        },
    }
}

async fn test_key_auth(server: &SshServerInfo) -> Result<(), String> {
    let mut args = build_ssh_args(server);
    // build_ssh_args 末尾是 destination (user@host),-o 选项必须插在
    // destination 之前,否则 SSH 把 -o 当作远程命令的一部分而非自身选项。
    let target = args.pop().unwrap();
    args.extend([
        "-o".into(),
        "BatchMode=yes".into(),
        "-o".into(),
        "ConnectTimeout=5".into(),
        "-o".into(),
        "StrictHostKeyChecking=no".into(),
        "-o".into(),
        "LogLevel=ERROR".into(),
    ]);
    args.push(target);
    args.push("echo ok".into());
    let cmd_args = args;

    match tokio::time::timeout(TEST_TIMEOUT, run_ssh_test(&cmd_args)).await {
        Ok(Ok(output)) => {
            // 严格匹配 `echo ok`,不放过 banner/motd 末尾恰好是 "ok" 的误判。
            if output.trim() == "ok" {
                Ok(())
            } else {
                Err(format!("Unexpected output: {}", output.trim()))
            }
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Connection timeout".into()),
    }
}

async fn test_password_auth(
    server: &SshServerInfo,
    password: Option<Zeroizing<String>>,
) -> Result<(), String> {
    let password = password.ok_or("Password not provided")?;

    // 构造 ssh 命令参数(注意 -o 选项必须插在 destination 之前,见该函数注释)
    let cmd_args = build_password_auth_cmd_args(server);

    // 平台分支:Windows 走 SSH_ASKPASS,其他平台走 stdin 注入
    #[cfg(windows)]
    return test_password_auth_windows(cmd_args, &password).await;
    #[cfg(not(windows))]
    test_password_auth_unix(cmd_args, &password).await
}

/// 非 Windows 平台:`ssh` 能从 pipe stdin 正常读密码。
#[cfg(not(windows))]
async fn test_password_auth_unix(
    cmd_args: Vec<String>,
    password: &Zeroizing<String>,
) -> Result<(), String> {
    let stdin_bytes = build_password_auth_stdin(password);

    let mut child = command::r#async::Command::new("ssh")
        .args(&cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 ssh 失败: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&stdin_bytes)
            .await
            .map_err(|e| format!("写入密码失败: {e}"))?;
    }

    let output = match tokio::time::timeout(TEST_TIMEOUT, child.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("读取 ssh 输出失败: {e}")),
        Err(_) => return Err("Connection timeout".into()),
    };

    finalize_password_test_result(&output)
}

/// Windows 平台:用 SSH_ASKPASS 机制把密码递给 ssh,完全绕开 stdin/控制台。
#[cfg(windows)]
async fn test_password_auth_windows(
    cmd_args: Vec<String>,
    password: &Zeroizing<String>,
) -> Result<(), String> {
    let askpass = AskpassSession::new(password).map_err(|e| format!("准备 askpass 失败: {e}"))?;

    let mut cmd = command::r#async::Command::new("ssh");
    cmd.args(&cmd_args)
        // ssh 不再需要从 stdin 读密码,设为 null 避免 ssh 误以为有 tty
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    askpass.apply_env(&mut cmd);

    let child = cmd.spawn().map_err(|e| format!("启动 ssh 失败: {e}"))?;

    // timeout 命中时 child 被 drop → kill_on_drop 自动 kill ssh。
    // askpass 守卫在函数尾部 drop,清理临时文件。
    let output = match tokio::time::timeout(TEST_TIMEOUT, child.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("读取 ssh 输出失败: {e}")),
        Err(_) => return Err("Connection timeout".into()),
    };
    drop(askpass);

    finalize_password_test_result(&output)
}

/// 解析 ssh 子进程的输出,统一成功/失败判定逻辑(两平台共享)。
fn finalize_password_test_result(output: &std::process::Output) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr_trimmed = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // 始终把 ssh 真实 stderr 落日志,即便成功也留痕,便于事后排查
    // "为什么 server 接受了 password 但 UI 报成功"的差异。
    if !stderr_trimmed.is_empty() {
        log::warn!("ssh test stderr: {stderr_trimmed}");
    }

    // 成功判定:严格匹配 `echo ok` 的输出。原先 `ends_with("ok")` 的兜底
    // 会让 banner / motd 末尾碰巧以 "ok" 结尾时误判为成功,这里去掉。
    if output.status.success() && stdout.trim() == "ok" {
        Ok(())
    } else if stderr_trimmed.contains("Permission denied")
        || stderr_trimmed.contains("Authentication failed")
    {
        // 错误信息带上精简 stderr(<= 200 字符),便于用户判断 server 端
        // 是没启 password、还是配置了 kbd-only AuthenticationMethods 等。
        let detail = if stderr_trimmed.is_empty() {
            String::new()
        } else {
            let snippet: String = stderr_trimmed.chars().take(200).collect();
            if stderr_trimmed.chars().count() > 200 {
                format!(" ({snippet}...)")
            } else {
                format!(" ({snippet})")
            }
        };
        Err(format!("Authentication failed: wrong password{detail}"))
    } else {
        Err(format!(
            "Unexpected output: stdout={} stderr={}",
            stdout.trim(),
            stderr_trimmed
        ))
    }
}

/// 把密码编码成要写入 ssh stdin 的字节流:密码 UTF-8 + 换行。
/// 独立成纯函数,便于单测断言"stdin 包含密码字面量 + 换行"。
/// 仅 unix 分支实际调用(Windows 走 SSH_ASKPASS),但函数本身跨平台编译,
/// 让 `build_password_auth_stdin_*` 单测可以在 Windows CI 上也跑。
// Windows 上仅测试调用此函数,生产路径用 SSH_ASKPASS,加 dead_code 抑制
#[cfg_attr(windows, allow(dead_code))]
fn build_password_auth_stdin(password: &Zeroizing<String>) -> Zeroizing<Vec<u8>> {
    let mut v = Zeroizing::new(Vec::with_capacity(password.len() + 1));
    v.extend_from_slice(password.as_bytes());
    v.push(b'\n');
    v
}

/// 拼出 password 认证测试时给 ssh 子进程的完整 argv。
///
/// 与 `build_ssh_args` 不同:这里跳过首项 `"ssh"`(我们用
/// `Command::new("ssh")` 显式派生),追加测试用 `-o` 选项和 `echo ok` 远端命令。
///
/// 关键选项含义:
/// - `BatchMode=no`:允许 ssh 从 stdin / askpass 读密码(不走 askpass 时需要 stdin)
/// - `PreferredAuthentications=password`:**只**声明想试 password,不带
///   `keyboard-interactive`。否则 server 端 PAM 在 password 之后会触发
///   kbd-interactive fallback,kbd-int 子 prompt 拿不到响应,会逐项重试
///   并触发 `pam_faildelay`(~2s/次),累计 ~8-10s 顶满 `TEST_TIMEOUT`。
/// - `KbdInteractiveAuthentication=no`:客户端能力开关,直接禁掉整个 kbd-int
///   协议。光靠 `PreferredAuthentications` 不够——它只约束 password 子方法的
///   prompt 次数,kbd-int 仍可走;两个开关都设才是 defense in depth。
/// - `NumberOfPasswordPrompts=1`:password 子方法只允许 1 次重试。
/// - `ConnectTimeout=5`:单次 TCP 连接超时。
/// - `StrictHostKeyChecking=no`:不拦 known_hosts(测试场景下避免 host key
///   变化导致误报,真实终端连接走别的路径)。
/// - `LogLevel=ERROR`:抑制 host key 提示 / banner 等噪音。
///
/// `echo ok` 作为远端命令,严格匹配 stdout 判定成功(避免 banner / motd
/// 末尾恰好含 "ok" 的误判)。
///
/// author: logic
/// date: 2026-06-01
fn build_password_auth_cmd_args(server: &SshServerInfo) -> Vec<String> {
    // skip(1) 去掉 "ssh" 本身(Command::new 已指定),剩下
    // ["-p","2222","user@host"]。-o 选项必须插在 destination 之前,
    // 否则 SSH 把 -o 当作远程命令的一部分而非自身选项。
    let mut args: Vec<String> = build_ssh_args(server).into_iter().skip(1).collect();
    let target = args.pop().unwrap();
    args.extend([
        "-o".into(),
        "BatchMode=no".into(),
        "-o".into(),
        "PreferredAuthentications=password".into(),
        "-o".into(),
        "KbdInteractiveAuthentication=no".into(),
        "-o".into(),
        "NumberOfPasswordPrompts=1".into(),
        "-o".into(),
        "ConnectTimeout=5".into(),
        "-o".into(),
        "StrictHostKeyChecking=no".into(),
        "-o".into(),
        "LogLevel=ERROR".into(),
    ]);
    args.push(target);
    args.push("echo ok".into());
    args
}

async fn run_ssh_test(args: &[String]) -> Result<String, std::io::Error> {
    // 统一走 command::r#async 派生子进程,Windows 上会带 CREATE_NO_WINDOW,
    // 避免闪出控制台窗口(见 .clippy.toml 对 tokio::process::Command 的禁用)。
    let output = command::r#async::Command::new(&args[0])
        .args(&args[1..])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // 成功判定:进程退出码为 0,或远端 `echo ok` 的输出已回传(部分 sshpass
    // 警告会让退出码非零,但 stdout 里仍含 "ok")。
    if output.status.success() || stdout.contains("ok") {
        Ok(stdout)
    } else {
        Err(std::io::Error::other(stderr))
    }
}

/// Windows 专属 askpass 会话:在临时目录创建密码文件 + askpass 辅助脚本,
/// 暴露给 `ssh` 通过 `SSH_ASKPASS` 环境变量使用,drop 时自动清理两个文件。
///
/// `ssh.exe` 在 Windows 上即便 stdin 是 pipe,也会因为无控制台而拒绝从
/// stdin 读密码(打印 `GetConsoleMode on STD_INPUT_HANDLE failed` 后挂死),
/// 详见 PowerShell/Win32-OpenSSH issue #1470。绕开方案是 `SSH_ASKPASS`:
/// `ssh` 看到该环境变量后,会派生指定程序并把它的 stdout 当作密码,完全
/// 绕过 stdin 和控制台。`SSH_ASKPASS_REQUIRE=force` 强制 ssh 即便检测到
/// TTY 也走 askpass 路径。
///
/// 密码通过临时文件传给 askpass 脚本(不写 env var,降低泄漏面):env var
/// 会在 `ssh` 子进程及其所有子进程里可见。askpass 进程生命周期极短(ssh
/// fork 后立刻 exec,读完就退出),落盘窗口可控到毫秒级。
///
/// **安全权衡**:两个临时文件不设 `FILE_ATTRIBUTE_HIDDEN`、不动 ACL,
/// 走 Windows `%TEMP%` 默认隔离(`C:\Users\<user>\AppData\Local\Temp`,
/// 每个用户独立)。早先版本试过隐藏属性 + icacls 收紧到 `(R)`,但
/// `FILE_ATTRIBUTE_HIDDEN` 会让 `posix_spawnp` 在 `CreateProcessW` 阶段
/// 返回 `ERROR_ACCESS_DENIED`(error 5),askpass 根本起不来,反而把
/// 密码错误地送到了 server 的 password prompt(用户看到 "wrong password"
/// 但其实根本没传出去)。Windows temp dir 的 per-user 隔离已经够用,
/// 这里把简单可靠排在"defense in depth"前面。
///
/// author: logic
/// date: 2026-06-01
#[cfg(windows)]
pub struct AskpassSession {
    password_path: std::path::PathBuf,
    script_path: std::path::PathBuf,
}

#[cfg(windows)]
impl AskpassSession {
    pub fn new(password: &Zeroizing<String>) -> std::io::Result<Self> {
        use std::io::Write as _;

        let dir = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let suffix = format!("{pid}-{nanos}");

        let password_path = dir.join(format!("warp-ssh-askpass-{suffix}.txt"));
        let script_path = dir.join(format!("warp-ssh-askpass-{suffix}.cmd"));

        // 写密码到临时文件(不设 hidden、不动 ACL,见类型 doc 的安全权衡)
        {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&password_path)?;
            f.write_all(password.as_bytes())?;
            f.sync_all()?;
        }

        // 写 askpass 辅助脚本:读取 %WARP_SSH_ASKPASS_FILE% 指向的文件首行,
        // echo 到 stdout。`set /p` 读首行(去掉换行),`echo !PW!` 输出。
        // 使用 `setlocal enabledelayedexpansion` + `!PW!` 延迟展开,避免密码
        // 含 cmd 特殊字符(&, |, <, >, ^)时被 %PW% 的即时展开二次解析截断。
        let body = "@echo off\r\nsetlocal enabledelayedexpansion\r\nset /p PW=<\"%WARP_SSH_ASKPASS_FILE%\"\r\necho !PW!\r\nendlocal\r\n";
        {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&script_path)?;
            f.write_all(body.as_bytes())?;
            f.sync_all()?;
        }

        Ok(Self {
            password_path,
            script_path,
        })
    }

    /// 把 SSH_ASKPASS 所需的环境变量挂到 ssh 子进程上。
    pub fn apply_env(&self, cmd: &mut command::r#async::Command) {
        cmd.env("SSH_ASKPASS", &self.script_path)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("WARP_SSH_ASKPASS_FILE", &self.password_path)
            .env_remove("DISPLAY");
    }

    pub fn apply_env_blocking(&self, cmd: &mut command::blocking::Command) {
        cmd.env("SSH_ASKPASS", &self.script_path)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("WARP_SSH_ASKPASS_FILE", &self.password_path)
            .env_remove("DISPLAY");
    }
}

#[cfg(windows)]
impl Drop for AskpassSession {
    fn drop(&mut self) {
        // ssh 退出后立即删除两个临时文件,缩短密码在磁盘上的存活窗口。
        // 错误吞掉:清理失败不应影响主流程返回值。
        let _ = std::fs::remove_file(&self.password_path);
        let _ = std::fs::remove_file(&self.script_path);
    }
}

#[cfg(test)]
#[path = "ssh_command_tests.rs"]
mod tests;
