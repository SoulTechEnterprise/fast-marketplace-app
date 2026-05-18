use self_update::cargo_crate_version;

const GITHUB_OWNER: &str = "SoulTechEnterprise";
const GITHUB_REPO: &str = "fast-marketplace-app";

pub fn check_and_update() {
    println!("🔍 Verificando atualizações...");

    let current_version = cargo_crate_version!();

    let bin_name = if cfg!(target_os = "windows") {
        "automatize-marketplace-windows.exe"
    } else if cfg!(target_os = "macos") {
        "automatize-marketplace-macos"
    } else {
        "automatize-marketplace-linux"
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner(GITHUB_OWNER)
        .repo_name(GITHUB_REPO)
        .bin_name(bin_name)
        .current_version(current_version)
        .no_confirm(true) // sem prompt para o usuário
        .show_output(false)
        .show_download_progress(true)
        .build();

    match status {
        Err(e) => {
            eprintln!("⚠️  Erro ao configurar updater: {}", e);
        }
        Ok(updater) => match updater.update() {
            Ok(status) => {
                if status.updated() {
                    println!(
                        "✅ App atualizado para v{}! Reiniciando...",
                        status.version()
                    );
                    // Reinicia o processo com os mesmos argumentos
                    restart_process();
                } else {
                    println!(
                        "✅ App já está na versão mais recente (v{}).",
                        current_version
                    );
                }
            }
            Err(self_update::errors::Error::Network(msg)) => {
                // Sem internet — continua normalmente
                eprintln!("⚠️  Sem conexão para verificar atualizações: {}", msg);
            }
            Err(e) => {
                eprintln!("⚠️  Falha na atualização: {}", e);
            }
        },
    }
}

fn restart_process() {
    let current_exe = std::env::current_exe().expect("Não foi possível obter o executável atual");
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut cmd = std::process::Command::new(current_exe);
    cmd.args(&args);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec(); // substitui o processo atual (sem fork)
        eprintln!("Falha ao reiniciar: {}", err);
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        cmd.spawn().expect("Falha ao reiniciar o processo");
        std::process::exit(0);
    }
}
