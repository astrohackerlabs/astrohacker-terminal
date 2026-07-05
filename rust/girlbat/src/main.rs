mod dispatch;
mod engine;
mod ffi;
mod ipc;
mod proto;
mod render_channel;

use std::sync::mpsc;

use proto::{Msg, TermSurfMessage};

#[derive(Debug)]
struct Args {
    ipc_socket: Option<String>,
    listen_socket: Option<String>,
    browser_name: String,
    user_data_dir: Option<String>,
    render_surface_service: Option<String>,
    incognito: bool,
    warmup: bool,
    abi_negative_smoke: bool,
    engine_thread_smoke: bool,
    render_surface_smoke: bool,
    real_frame_attachment_smoke: bool,
    renderer_crash_smoke: bool,
    resource_root_smoke: bool,
}

fn main() {
    let args = parse_args();

    if args.warmup {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = ffi::warmup();
        eprintln!("[Girlbat] warmup runtime={runtime} version={version} ok={ok}");
        return;
    }

    if args.abi_negative_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = ffi::negative_smoke();
        eprintln!("[Girlbat] abi-negative-smoke runtime={runtime} version={version} ok={ok}");
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if args.engine_thread_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = engine::owner_thread_smoke();
        eprintln!("[Girlbat] engine-thread-smoke runtime={runtime} version={version} ok={ok}");
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if args.render_surface_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = ffi::render_surface_smoke();
        eprintln!("[Girlbat] render-surface-smoke runtime={runtime} version={version} ok={ok}");
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if args.real_frame_attachment_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = ffi::real_frame_attachment_smoke();
        eprintln!(
            "[Girlbat] real-frame-attachment-smoke runtime={runtime} version={version} ok={ok}"
        );
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if args.renderer_crash_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let ok = ffi::renderer_crash_smoke();
        eprintln!("[Girlbat] renderer-crash-smoke runtime={runtime} version={version} ok={ok}");
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    if args.resource_root_smoke {
        let runtime = ffi::runtime_name();
        let version = ffi::runtime_version();
        let resource_root = ffi::runtime_resource_root();
        println!("{resource_root}");
        eprintln!(
            "[Girlbat] resource-root-smoke runtime={runtime} version={version} root={resource_root}"
        );
        if resource_root.is_empty() {
            std::process::exit(1);
        }
        return;
    }

    eprintln!(
        "[Girlbat] starting browser={} user_data_dir={} incognito={} render_surface_service_configured={} abi_runtime={} abi_version={}",
        args.browser_name,
        args.user_data_dir.as_deref().unwrap_or(""),
        args.incognito,
        args.render_surface_service.is_some(),
        ffi::runtime_name(),
        ffi::runtime_version()
    );

    if let Some(service) = args.render_surface_service.as_deref() {
        let connected = render_channel::connect_global(service);
        eprintln!("[Girlbat] render side-channel global connected={connected}");
    }

    if let Err(error) = engine::init_global() {
        eprintln!("[Girlbat] engine startup failed: {error}");
        std::process::exit(1);
    }

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let mut active_transport = false;

    if let Some(path) = args.ipc_socket.as_deref() {
        match ipc::connect(path) {
            Some(reader) => {
                active_transport = true;
                let msg = TermSurfMessage {
                    msg: Some(Msg::ServerRegister(proto::termsurf::ServerRegister {
                        profile: profile_name(args.user_data_dir.as_deref()),
                        browser: args.browser_name.clone(),
                    })),
                };
                ipc::send(&msg);
                ipc::spawn_reader(reader, true, shutdown_tx.clone());
            }
            None => {
                eprintln!("[Girlbat] unable to connect to --ipc-socket={path}");
            }
        }
    }

    if let Some(path) = args.listen_socket.as_deref() {
        match ipc::listen(path, shutdown_tx.clone()) {
            Ok(()) => active_transport = true,
            Err(error) => {
                eprintln!("[Girlbat] listener startup failed: {error}");
                if !active_transport {
                    engine::shutdown_global();
                    std::process::exit(1);
                }
            }
        }
    }

    if args.ipc_socket.is_none() && args.listen_socket.is_none() {
        eprintln!("[Girlbat] no --ipc-socket or --listen-socket supplied; exiting");
        engine::shutdown_global();
        return;
    }

    if !active_transport {
        eprintln!("[Girlbat] no active transport after startup; exiting");
        engine::shutdown_global();
        std::process::exit(1);
    }

    match shutdown_rx.recv() {
        Ok(reason) => eprintln!("[Girlbat] shutting down: {reason}"),
        Err(_) => eprintln!("[Girlbat] shutting down: channel closed"),
    }
    engine::shutdown_global();
}

fn parse_args() -> Args {
    parse_args_from(std::env::args().skip(1))
}

fn parse_args_from<I, S>(raw_args: I) -> Args
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = Args {
        ipc_socket: None,
        listen_socket: None,
        browser_name: "girlbat".to_string(),
        user_data_dir: None,
        render_surface_service: None,
        incognito: false,
        warmup: false,
        abi_negative_smoke: false,
        engine_thread_smoke: false,
        render_surface_smoke: false,
        real_frame_attachment_smoke: false,
        renderer_crash_smoke: false,
        resource_root_smoke: false,
    };

    for raw_arg in raw_args {
        let arg = raw_arg.as_ref();
        if let Some(value) = arg.strip_prefix("--ipc-socket=") {
            args.ipc_socket = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--listen-socket=") {
            args.listen_socket = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--browser-name=") {
            args.browser_name = value.to_string();
        } else if let Some(value) = arg.strip_prefix("--user-data-dir=") {
            args.user_data_dir = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--render-surface-service=") {
            args.render_surface_service = Some(value.to_string());
        } else if arg == "--incognito" {
            args.incognito = true;
        } else if arg == "--termsurf-warmup" {
            args.warmup = true;
        } else if arg == "--termsurf-abi-negative-smoke" {
            args.abi_negative_smoke = true;
        } else if arg == "--termsurf-engine-thread-smoke" {
            args.engine_thread_smoke = true;
        } else if arg == "--termsurf-render-surface-smoke" {
            args.render_surface_smoke = true;
        } else if arg == "--termsurf-real-frame-attachment-smoke" {
            args.real_frame_attachment_smoke = true;
        } else if arg == "--termsurf-renderer-crash-smoke" {
            args.renderer_crash_smoke = true;
        } else if arg == "--termsurf-resource-root-smoke" {
            args.resource_root_smoke = true;
        } else if arg == "--hidden" || arg == "--no-sandbox" || arg == "--enable-logging" {
            eprintln!("[Girlbat] accepted compatibility flag: {arg}");
        } else if let Some(value) = arg.strip_prefix("--log-file=") {
            eprintln!("[Girlbat] accepted compatibility log file: {value}");
        } else {
            eprintln!("[Girlbat] ignoring unknown argument: {arg}");
        }
    }

    args
}

fn profile_name(user_data_dir: Option<&str>) -> String {
    user_data_dir
        .and_then(|path| std::path::Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("default")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_render_surface_service_arg() {
        let args = parse_args_from([
            "--render-surface-service=com.astrohacker.terminal.ladybirdd.render.123.default.girlbat",
        ]);

        assert_eq!(
            args.render_surface_service.as_deref(),
            Some("com.astrohacker.terminal.ladybirdd.render.123.default.girlbat")
        );
    }

    #[test]
    fn parse_existing_launch_and_compatibility_args() {
        let args = parse_args_from([
            "--ipc-socket=/tmp/termsurf-gui.sock",
            "--listen-socket=/tmp/girlbat.sock",
            "--browser-name=girlbat",
            "--user-data-dir=/tmp/termsurf/browser-profiles/default",
            "--incognito",
            "--termsurf-warmup",
            "--termsurf-abi-negative-smoke",
            "--termsurf-engine-thread-smoke",
            "--termsurf-render-surface-smoke",
            "--termsurf-real-frame-attachment-smoke",
            "--termsurf-resource-root-smoke",
            "--hidden",
            "--no-sandbox",
            "--enable-logging",
            "--log-file=/tmp/girlbat.log",
        ]);

        assert_eq!(args.ipc_socket.as_deref(), Some("/tmp/termsurf-gui.sock"));
        assert_eq!(args.listen_socket.as_deref(), Some("/tmp/girlbat.sock"));
        assert_eq!(args.browser_name, "girlbat");
        assert_eq!(
            args.user_data_dir.as_deref(),
            Some("/tmp/termsurf/browser-profiles/default")
        );
        assert_eq!(args.render_surface_service, None);
        assert!(args.incognito);
        assert!(args.warmup);
        assert!(args.abi_negative_smoke);
        assert!(args.engine_thread_smoke);
        assert!(args.render_surface_smoke);
        assert!(args.real_frame_attachment_smoke);
        assert!(args.resource_root_smoke);
    }
}
