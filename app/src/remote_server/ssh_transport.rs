//! SSH-specific implementation of [`RemoteTransport`].
//!
//! [`SshTransport`] uses an existing SSH ControlMaster socket to check/install
//! the remote server binary and to launch the `remote-server-proxy` process
//! whose stdin/stdout become the protocol channel.
use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use warpui::r#async::{executor, FutureExt as _};

use remote_server::auth::RemoteServerAuthContext;
use remote_server::client::RemoteServerClient;
use remote_server::setup::{
    parse_uname_output, remote_server_daemon_dir, PreinstallCheckResult, RemotePlatform,
};
use remote_server::ssh::ssh_args;
use remote_server::transport::{Connection, RemoteTransport};

/// SSH transport: connects via a ControlMaster socket.
///
/// `socket_path` is the local Unix socket created by the ControlMaster
/// process (`ssh -N -o ControlMaster=yes -o ControlPath=<path>`). All SSH
/// commands (binary check, install, proxy launch) are multiplexed through
/// this socket without re-authenticating.
#[derive(Clone)]
pub struct SshTransport {
    socket_path: PathBuf,
    auth_context: Arc<RemoteServerAuthContext>,
}

impl fmt::Debug for SshTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SshTransport")
            .field("socket_path", &self.socket_path)
            .finish_non_exhaustive()
    }
}

impl SshTransport {
    pub fn new(socket_path: PathBuf, auth_context: Arc<RemoteServerAuthContext>) -> Self {
        Self {
            socket_path,
            auth_context,
        }
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    pub fn remote_daemon_socket_path(&self) -> String {
        format!(
            "{}/server.sock",
            remote_server_daemon_dir(&self.auth_context.remote_server_identity_key())
        )
    }

    pub fn remote_daemon_pid_path(&self) -> String {
        format!(
            "{}/server.pid",
            remote_server_daemon_dir(&self.auth_context.remote_server_identity_key())
        )
    }

    fn remote_proxy_command(&self) -> String {
        let binary = remote_server::setup::remote_server_binary();
        let identity_key = self.auth_context.remote_server_identity_key();
        let quoted_identity_key = shell_words::quote(&identity_key);
        format!("{binary} remote-server-proxy --identity-key {quoted_identity_key}")
    }
}

#[derive(Debug)]
enum InstallError {
    ScriptFailed { exit_code: i32, stderr: String },
    Other(anyhow::Error),
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScriptFailed { exit_code, stderr } => {
                write!(f, "install script failed (exit {exit_code}): {stderr}")
            }
            Self::Other(error) => write!(f, "{error:#}"),
        }
    }
}

impl From<anyhow::Error> for InstallError {
    fn from(error: anyhow::Error) -> Self {
        Self::Other(error)
    }
}

async fn detect_remote_platform(socket_path: &Path) -> Result<RemotePlatform> {
    let output = remote_server::ssh::run_ssh_command(
        socket_path,
        "uname -sm",
        remote_server::setup::CHECK_TIMEOUT,
    )
    .await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return parse_uname_output(&stdout);
    }

    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(anyhow!("uname -sm exited with code {code}: {stderr}"))
}

async fn verify_installed_binary(socket_path: &Path) -> Result<()> {
    let output = remote_server::ssh::run_ssh_command(
        socket_path,
        &remote_server::setup::binary_check_command(),
        remote_server::setup::CHECK_TIMEOUT,
    )
    .await?;

    if output.status.success() {
        return Ok(());
    }

    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(anyhow!(
        "installed binary check failed with code {code}: {stderr}"
    ))
}

async fn run_install_script(
    socket_path: &Path,
    staging_tarball_path: Option<&str>,
    timeout: std::time::Duration,
) -> core::result::Result<(), InstallError> {
    let script = remote_server::setup::install_script(staging_tarball_path);
    match remote_server::ssh::run_ssh_script(socket_path, &script, timeout).await {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(InstallError::ScriptFailed { exit_code, stderr })
        }
        Err(error) => Err(InstallError::Other(error)),
    }
}

fn should_skip_scp_fallback(error: &InstallError) -> bool {
    matches!(error, InstallError::ScriptFailed { exit_code: 2, .. })
}

// ===========================================================================
// Zap fork:开发模式 remote-server 安装路径
//
// 上游 / release 构建会让远端安装脚本从 GitHub releases 下载预编译的
// remote-server 二进制。但在本地源码构建(`cargo run`)时,这会下载到
// 「最新已发布」的陈旧二进制,而不是开发者刚改过的代码,导致根本无法
// 调试 remote-server 的改动。
//
// 因此在 DEBUG 且无 release tag 的源码构建下(见
// `remote_server::setup::is_dev_source_build()`),`install_binary()` 改为:
//   1. 本地把 `warp` 二进制交叉编译到 x86_64 musl(profile/features 与
//      `script/deploy_remote_server` 完全一致);
//   2. 通过已有的 SSH ControlMaster socket,用 `scp_upload` 把产物上传到
//      `remote_server::setup::remote_server_binary()` 解析出的远端路径;
//   3. 完全跳过 GitHub 下载安装脚本。
//
// 如果交叉编译前置条件缺失(没装 musl target、没有 musl 链接器),不会
// 硬失败,而是打印清晰告警并回退到原有下载安装流程,保证 dev 仍可用。
// ===========================================================================

/// 开发模式交叉编译可能用到的 musl 链接器候选(按优先级)。
/// macOS 上一般是 `x86_64-linux-musl-gcc`(filosottile/musl-cross),
/// Linux 上常见为 `musl-gcc`。
const DEV_MUSL_LINKER_CANDIDATES: &[&str] = &["x86_64-linux-musl-gcc", "musl-gcc"];

/// 返回当前 workspace 根目录。
///
/// `ssh_transport.rs` 属于 `app` crate,`CARGO_MANIFEST_DIR` 指向
/// `<workspace>/app`,其父目录即 workspace 根。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        // 理论上 `app` 一定有父目录;万一没有就退回 manifest 目录本身。
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

/// 返回追加了 `~/.cargo/bin`(及 `$CARGO_HOME/bin`)的 PATH。
///
/// warp 进程常由桌面环境或系统 `cargo` 拉起,其 PATH 可能只含 `/usr/bin`
/// 而不含 `~/.cargo/bin`。这会导致:
///   - `cargo zigbuild` 找不到 `cargo-zigbuild` 子命令 → 回退到 musl-gcc;
///   - cargo-zigbuild 自身找不到 `cargo` / `rustc`。
/// 交叉编译相关的子进程统一用这里返回的 PATH,保证两者都能解析到。
/// 若无需调整(无 HOME / 无法拼接)返回 `None`,调用方沿用继承的 PATH。
fn dev_build_path_env() -> Option<std::ffi::OsString> {
    let mut extra: Vec<PathBuf> = Vec::new();
    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        extra.push(PathBuf::from(cargo_home).join("bin"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        extra.push(PathBuf::from(home).join(".cargo").join("bin"));
    }
    if extra.is_empty() {
        return None;
    }
    let current = std::env::var_os("PATH").unwrap_or_default();
    extra.extend(std::env::split_paths(&current));
    std::env::join_paths(extra).ok()
}

/// 在 `PATH` 中查找首个可用的 musl 链接器,找不到返回 `None`。
fn find_musl_linker() -> Option<&'static str> {
    DEV_MUSL_LINKER_CANDIDATES.iter().copied().find(|linker| {
        command::blocking::Command::new(linker)
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    })
}

/// dev 交叉编译使用的构建后端。
enum DevBuildBackend {
    /// `cargo zigbuild`:zig 充当完整的 C/C++ musl 交叉工具链,无需单独安装
    /// `*-musl-gcc` / `*-musl-g++`,能正确编译 `freetype-sys` 等带 C/C++ 源码
    /// 的依赖。这是首选后端。
    Zigbuild,
    /// 原生 `cargo build` + musl 链接器。仅当系统装有完整的 musl C/C++ 交叉
    /// 工具链时才可靠 —— 只有 `*-musl-gcc`、缺 `*-musl-g++` 时,`freetype-sys`
    /// 之类的 C++ 依赖会编译失败。
    MuslGcc(&'static str),
}

/// 检测 `cargo-zigbuild` 是否可用。
///
/// 直接探测 `cargo-zigbuild --version`(二进制本身),而不是
/// `cargo zigbuild --version` —— 后者会被 `zigbuild` 子命令解析为未知参数
/// 而失败。探测用的 PATH 与实际构建一致(注入 `~/.cargo/bin`)。
fn cargo_zigbuild_available() -> bool {
    let mut cmd = command::blocking::Command::new("cargo-zigbuild");
    cmd.arg("--version");
    if let Some(path) = dev_build_path_env() {
        cmd.env("PATH", path);
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// 选择 dev 交叉编译后端:优先 `cargo zigbuild`,回退到原生 `cargo build`
/// + musl 链接器。两者都不可用时返回 `None`,由调用方回退到下载安装。
fn select_dev_build_backend() -> Option<DevBuildBackend> {
    if cargo_zigbuild_available() {
        return Some(DevBuildBackend::Zigbuild);
    }
    find_musl_linker().map(DevBuildBackend::MuslGcc)
}

/// 检查 `x86_64-unknown-linux-musl` target 是否已通过 rustup 安装。
async fn musl_target_installed() -> bool {
    let output = command::r#async::Command::new("rustup")
        .arg("target")
        .arg("list")
        .arg("--installed")
        .kill_on_drop(true)
        .output()
        .await;
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim() == remote_server::setup::DEV_MUSL_TARGET),
        // 拿不到 rustup 输出时保守地认为未安装,从而触发回退。
        _ => false,
    }
}

/// 交叉编译本地 `warp` 二进制到 musl,返回产物路径。
///
/// profile / features 与 `script/deploy_remote_server` 对齐。
async fn cross_compile_remote_server(backend: &DevBuildBackend) -> Result<PathBuf> {
    let root = workspace_root();
    // 当前 channel 对应的 `[[bin]]` 名 —— OSS fork 是 `warp-oss`(见 app/Cargo.toml)。
    // 不能写死 `warp`:`warp` 那个 bin 走 `load_config!("local")`,需要私有的
    // `warp-channel-config` 才能生成 `local_config.json`,OSS fork 没有它会编译失败;
    // `warp-oss`(src/bin/oss.rs)内联 `ChannelConfig`,无此依赖。
    let bin_name = remote_server::setup::binary_name();
    let backend_desc = match backend {
        DevBuildBackend::Zigbuild => "cargo-zigbuild".to_string(),
        DevBuildBackend::MuslGcc(linker) => format!("cargo-build/{linker}"),
    };
    log::info!(
        "dev remote-server: 交叉编译 {bin_name} -> {} (profile={}, backend={backend_desc})",
        remote_server::setup::DEV_MUSL_TARGET,
        remote_server::setup::DEV_REMOTE_PROFILE,
    );
    // 首次会编译整个 warp,耗时通常数分钟。stdout/stderr 直接 inherit 到运行
    // Zap 的终端,这样开发者能看到 cargo 的实时编译进度(否则全程静默,
    // 容易误以为卡死)。
    log::info!(
        "dev remote-server: 正在交叉编译,首次通常需数分钟 —— cargo 进度会打印到\
         运行 Zap 的终端"
    );

    let status = async {
        let mut cmd = command::r#async::Command::new("cargo");
        cmd.current_dir(&root);
        // 注入 `~/.cargo/bin`,确保 `cargo zigbuild` 能解析 `cargo-zigbuild`
        // 子命令,且 cargo-zigbuild 能找到 `cargo` / `rustc`。
        if let Some(path) = dev_build_path_env() {
            cmd.env("PATH", path);
        }
        match backend {
            // zigbuild 是 cargo 子命令,自带 zig 链接器与 C/C++ 交叉编译器,
            // 无需再设 LINKER env。
            DevBuildBackend::Zigbuild => {
                cmd.arg("zigbuild");
            }
            // 原生 cargo build:通过 env 指定 musl 链接器并覆盖 rustflags,
            // 避免 .cargo/config.toml 里 macOS 专用 flag 污染交叉编译。
            DevBuildBackend::MuslGcc(linker) => {
                cmd.arg("build")
                    .env("CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER", *linker)
                    .env(
                        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS",
                        "-C symbol-mangling-version=v0",
                    );
            }
        }
        cmd.arg("-p")
            .arg("warp")
            .arg("--bin")
            .arg(bin_name)
            .arg("--target")
            .arg(remote_server::setup::DEV_MUSL_TARGET)
            .arg("--profile")
            .arg(remote_server::setup::DEV_REMOTE_PROFILE)
            .arg("--features")
            .arg(remote_server::setup::DEV_REMOTE_FEATURES)
            // inherit:把 cargo 实时进度透到终端,而不是全程静默缓冲。
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true)
            .status()
            .await
    }
    .with_timeout(remote_server::setup::DEV_CROSS_COMPILE_TIMEOUT)
    .await
    .map_err(|_| {
        anyhow!(
            "dev remote-server cross-compilation timed out (>{:?})",
            remote_server::setup::DEV_CROSS_COMPILE_TIMEOUT
        )
    })?
    .map_err(|e| anyhow!("Could not start the cargo build: {e}"))?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(anyhow!(
            "cargo cross-compilation failed (exit {code}); see the cargo output in the terminal running Zap"
        ));
    }

    // 产物位置:`<target_dir>/<triple>/<profile>/<bin_name>`。
    // 优先读 `CARGO_TARGET_DIR`,否则回退到 `<workspace>/target`。仓库未在
    // `.cargo/config.toml` 里设 `[build] target-dir`,故只需考虑 env。
    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let binary = target_root
        .join(remote_server::setup::DEV_MUSL_TARGET)
        .join(remote_server::setup::DEV_REMOTE_PROFILE)
        .join(bin_name);
    if !binary.is_file() {
        return Err(anyhow!(
            "Cross-compilation finished but no artifact was found at {} (if CARGO_TARGET_DIR is set, verify the path)",
            binary.display()
        ));
    }
    Ok(binary)
}

/// 开发模式安装:交叉编译本地 `warp` 并上传到远端 remote-server 路径。
///
/// 上传目标与 `remote_server_binary()` 完全一致,确保随后的
/// `check_binary()` / proxy 启动能找到它。
async fn dev_install_local_binary(socket_path: &Path) -> Result<()> {
    // 前置条件检查:缺任意一项都返回错误,由调用方回退到下载安装。
    if !musl_target_installed().await {
        return Err(anyhow!(
            "The rust target {} is not installed; run `rustup target add {}`",
            remote_server::setup::DEV_MUSL_TARGET,
            remote_server::setup::DEV_MUSL_TARGET,
        ));
    }
    // 选择交叉编译后端:优先 `cargo zigbuild`(zig 自带完整 C/C++ musl 工具链,
    // 能编译 freetype-sys 等 C++ 依赖),否则回退到 musl-gcc。两者皆无则报错。
    let backend = select_dev_build_backend().ok_or_else(|| {
        anyhow!(
            "No usable musl cross-compilation backend was found. Install cargo-zigbuild + zig \
             (`cargo install cargo-zigbuild`, plus `zig` from your package manager), \
             or install a complete musl C/C++ cross toolchain ({})",
            DEV_MUSL_LINKER_CANDIDATES.join(" / ")
        )
    })?;

    let local_binary = cross_compile_remote_server(&backend).await?;

    // 上传到 `remote_server_binary()` 解析出的精确路径,先建好父目录。
    let remote_binary = remote_server::setup::remote_server_binary();
    let remote_dir = remote_server::setup::remote_server_dir();
    let mkdir_output = remote_server::ssh::run_ssh_command(
        socket_path,
        &format!("mkdir -p {remote_dir}"),
        remote_server::setup::CHECK_TIMEOUT,
    )
    .await?;
    if !mkdir_output.status.success() {
        let code = mkdir_output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&mkdir_output.stderr);
        return Err(anyhow!(
            "Failed to create the remote remote-server directory (exit {code}): {stderr}"
        ));
    }

    log::info!("dev remote-server: 上传本地交叉编译产物到 {remote_binary}(scp -C 压缩,数百 MB 可能需数分钟)");
    // dev 产物有数百 MB,用 DEV_UPLOAD_TIMEOUT(远超 SCP_INSTALL_TIMEOUT),
    // 避免大文件上传被 120s 超时打断后回退到下载陈旧 release。
    remote_server::ssh::scp_upload(
        socket_path,
        &local_binary,
        &remote_binary,
        remote_server::setup::DEV_UPLOAD_TIMEOUT,
    )
    .await?;

    // 赋予可执行权限。
    let chmod_output = remote_server::ssh::run_ssh_command(
        socket_path,
        &format!("chmod 755 {remote_binary}"),
        remote_server::setup::CHECK_TIMEOUT,
    )
    .await?;
    if !chmod_output.status.success() {
        let code = chmod_output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&chmod_output.stderr);
        return Err(anyhow!("Remote chmod failed (exit {code}): {stderr}"));
    }

    // 复用既有校验逻辑确认上传的二进制可运行。
    verify_installed_binary(socket_path).await
}

async fn download_remote_server_tarball(download_url: &str, tarball_path: &Path) -> Result<()> {
    let output = async {
        command::r#async::Command::new("curl")
            .arg("-fSL")
            .arg("--connect-timeout")
            .arg("15")
            .arg(download_url)
            .arg("-o")
            .arg(tarball_path.as_os_str())
            .kill_on_drop(true)
            .output()
            .await
    }
    .with_timeout(remote_server::setup::SCP_INSTALL_TIMEOUT)
    .await
    .map_err(|_| {
        anyhow!(
            "local tarball download timed out after {:?}",
            remote_server::setup::SCP_INSTALL_TIMEOUT
        )
    })?
    .map_err(|e| anyhow!("local curl failed to execute: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(anyhow!(
        "local tarball download failed with code {code}: {stderr}"
    ))
}

async fn scp_install_fallback(socket_path: &Path) -> Result<()> {
    let platform = detect_remote_platform(socket_path).await?;
    let download_url = remote_server::setup::download_tarball_url(&platform);
    let remote_server_dir = remote_server::setup::remote_server_dir();
    let mkdir_cmd = format!("mkdir -p {remote_server_dir}");
    let mkdir_output = remote_server::ssh::run_ssh_command(
        socket_path,
        &mkdir_cmd,
        remote_server::setup::CHECK_TIMEOUT,
    )
    .await?;

    if !mkdir_output.status.success() {
        let code = mkdir_output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&mkdir_output.stderr);
        return Err(anyhow!(
            "remote-server dir creation failed with code {code}: {stderr}"
        ));
    }

    let tempdir = tempfile::tempdir()?;
    let tarball_path = tempdir.path().join("zap.tar.gz");
    download_remote_server_tarball(&download_url, &tarball_path).await?;

    let remote_tarball_path = format!("{remote_server_dir}/zap-upload.tar.gz");
    remote_server::ssh::scp_upload(
        socket_path,
        &tarball_path,
        &remote_tarball_path,
        remote_server::setup::SCP_INSTALL_TIMEOUT,
    )
    .await?;

    run_install_script(
        socket_path,
        Some(&remote_tarball_path),
        remote_server::setup::SCP_INSTALL_TIMEOUT,
    )
    .await
    .map_err(|error| anyhow!("staged install failed: {error}"))?;

    verify_installed_binary(socket_path).await
}

impl RemoteTransport for SshTransport {
    fn detect_platform(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<RemotePlatform, String>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            detect_remote_platform(&socket_path)
                .await
                .map_err(|e| format!("{e:#}"))
        })
    }

    fn run_preinstall_check(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<PreinstallCheckResult, String>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            match remote_server::ssh::run_ssh_script(
                &socket_path,
                remote_server::setup::PREINSTALL_CHECK_SCRIPT,
                remote_server::setup::CHECK_TIMEOUT,
            )
            .await
            {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(PreinstallCheckResult::parse(&stdout))
                }
                Ok(output) => {
                    let code = output.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!(
                        "Preinstall check exited with code {code}: {stderr}"
                    ))
                }
                Err(e) => Err(format!("{e:#}")),
            }
        })
    }

    fn check_binary(&self) -> Pin<Box<dyn Future<Output = Result<bool, String>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            let bin_path = remote_server::setup::remote_server_binary();
            log::info!("Checking for remote server binary at {bin_path}");
            match remote_server::ssh::run_ssh_command(
                &socket_path,
                &remote_server::setup::binary_check_command(),
                remote_server::setup::CHECK_TIMEOUT,
            )
            .await
            {
                // `{binary} --version` 退出 0 表示存在且可运行。
                // 126/127 表示缺失或不可执行;其他非 0 退出视为真实检查失败。
                Ok(output) => match output.status.code() {
                    Some(0) => Ok(true),
                    Some(126) | Some(127) => Ok(false),
                    Some(code) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(format!("binary check exited with code {code}: {stderr}"))
                    }
                    None => Err("binary check terminated by signal".into()),
                },
                Err(e) => Err(format!("{e:#}")),
            }
        })
    }

    fn check_has_old_binary(&self) -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            // Treat the existence of the remote-server install directory
            // itself as evidence of a prior install. If `~/.warp-XX/remote-server`
            // exists, something was installed there before, so any mismatch
            // with the client's expected binary path should be auto-updated
            // rather than surfaced as a first-time install prompt.
            let cmd = format!("test -d {}", remote_server::setup::remote_server_dir());
            let output = remote_server::ssh::run_ssh_command(
                &socket_path,
                &cmd,
                remote_server::setup::CHECK_TIMEOUT,
            )
            .await?;
            // `test -d` exits 0 when present, 1 when missing.
            // Anything else is treated as a check failure.
            match output.status.code() {
                Some(0) => Ok(true),
                Some(1) => Ok(false),
                Some(code) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(anyhow::anyhow!(
                        "remote-server dir check exited with code {code}: {stderr}"
                    ))
                }
                None => Err(anyhow::anyhow!(
                    "remote-server dir check terminated by signal"
                )),
            }
        })
    }

    fn install_binary(&self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            log::info!(
                "Installing remote server binary to {}",
                remote_server::setup::remote_server_binary()
            );

            // Zap fork:DEBUG 源码构建(无 release tag)走开发模式,
            // 交叉编译本地 `warp` 并上传,而不是下载陈旧的 GitHub release。
            // 失败时(交叉编译前置条件缺失等)打印告警并回退到下载安装,
            // 保证 dev 体验不被破坏。release 构建跳过整段逻辑,行为不变。
            if remote_server::setup::is_dev_source_build() {
                log::info!("dev remote-server: 检测到 DEBUG 源码构建,改用本地交叉编译安装");
                match dev_install_local_binary(&socket_path).await {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        log::warn!(
                            "dev remote-server: 本地交叉编译安装不可用,回退到下载安装: {error:#}"
                        );
                        // 落空,继续走下方常规下载安装流程。
                    }
                }
            }

            match run_install_script(&socket_path, None, remote_server::setup::INSTALL_TIMEOUT)
                .await
            {
                Ok(()) => verify_installed_binary(&socket_path)
                    .await
                    .map_err(|error| format!("{error:#}")),
                Err(error) if should_skip_scp_fallback(&error) => Err(error.to_string()),
                Err(error) => {
                    log::warn!("remote-server install failed, trying SCP fallback: {error}");
                    match scp_install_fallback(&socket_path).await {
                        Ok(()) => Ok(()),
                        Err(fallback_error) => {
                            Err(format!("{error}; SCP fallback failed: {fallback_error:#}"))
                        }
                    }
                }
            }
        })
    }

    fn connect(
        &self,
        executor: Arc<executor::Background>,
    ) -> Pin<Box<dyn Future<Output = Result<Connection>> + Send>> {
        let socket_path = self.socket_path.clone();
        let remote_proxy_command = self.remote_proxy_command();
        Box::pin(async move {
            let mut args = ssh_args(&socket_path);
            args.push(remote_proxy_command);

            // `kill_on_drop(true)` pairs with ownership of the `Child` being
            // returned in the [`Connection`] below: the
            // [`RemoteServerManager`] holds the `Child` on its per-session
            // state, and dropping that state (on explicit teardown or
            // spontaneous disconnect) sends SIGKILL to this ssh process.
            let mut child = command::r#async::Command::new("ssh")
                .args(&args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;

            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture child stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture child stdout"))?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture child stderr"))?;

            let (client, event_rx) =
                RemoteServerClient::from_child_streams(stdin, stdout, stderr, &executor);
            Ok(Connection {
                client,
                event_rx,
                child,
                control_path: Some(socket_path),
            })
        })
    }

    fn remove_remote_server_binary(
        &self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            let cmd = format!("rm -f {}", remote_server::setup::remote_server_binary());
            log::info!("Removing stale remote server binary: {cmd}");
            let output = remote_server::ssh::run_ssh_command(
                &socket_path,
                &cmd,
                remote_server::setup::CHECK_TIMEOUT,
            )
            .await?;
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to remove binary: {stderr}"))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warpui::r#async::BoxFuture;
    fn static_auth_context() -> Arc<RemoteServerAuthContext> {
        Arc::new(RemoteServerAuthContext::new(
            || -> BoxFuture<'static, Option<String>> { Box::pin(async { None }) },
            || "user id/with spaces".to_string(),
        ))
    }

    #[test]
    fn remote_proxy_command_quotes_identity_key() {
        let transport = SshTransport::new(
            PathBuf::from("/tmp/control-master.sock"),
            static_auth_context(),
        );

        let command = transport.remote_proxy_command();

        assert!(command.contains("remote-server-proxy --identity-key"));
        assert!(command.contains("'user id/with spaces'"));
    }
}
