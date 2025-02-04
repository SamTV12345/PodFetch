#[cfg(test)]
pub mod test {
    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex};
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread::spawn;
    use bollard::container::{ListContainersOptions, RemoveContainerOptions};
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;
    use chrono::Utc;
    use ctor::{ctor, dtor};
    use diesel::RunQueryDsl;
    use sha256::digest;
    use testcontainers::{Container, ContainerRequest, ImageExt};
    use testcontainers::core::ContainerPort;
    use testcontainers::runners::{SyncRunner};
    use testcontainers_modules::postgres::Postgres;
    use bollard::Docker;

    #[derive(Debug)]
    pub enum ContainerCommands {
        Stop,
        Cleanup
    }

    pub struct Channel<T> {
        pub tx: Sender<T>,
        rx: Mutex<Receiver<T>>,
    }

    pub static POSTGRES_CHANNEL: LazyLock<Channel<ContainerCommands>> = LazyLock::new(||{
       let (tx, rx) = std::sync::mpsc::channel();
         Channel {
              tx,
              rx: Mutex::new(rx),
         }
    });

    #[dtor]
    fn on_shutdown() {
        POSTGRES_CHANNEL.tx.send(ContainerCommands::Stop).unwrap()
    }

    #[ctor]
    fn on_startup() {
        let docker = Docker::connect_with_defaults().unwrap();
        let container = setup_container();
        let running_container = match SyncRunner::start(container) {
            Ok(container)=> Ok::<Container<Postgres>, String>(container),
            Err(_)=>{
                let runner = tokio::runtime::Builder::new_multi_thread()
                    .thread_name("testcontainers-worker")
                    .worker_threads(2)
                    .enable_all()
                    .build().unwrap();
                let options = Some(ListContainersOptions::<String>{
                    all: true,
                    filters: HashMap::new(),
                    ..Default::default()
                });
                for container in runner.block_on(docker.list_containers(options)).unwrap() {
                    if let Some(port) = container.ports.clone() {
                        for p in port {
                            if p.public_port == Some(55002) {
                                println!("Removing container: {:?}", container.clone());
                                if let Some(id) = container.id.clone() {
                                    let docker = Docker::connect_with_defaults().unwrap();
                                    let options = Some(RemoveContainerOptions{
                                        force: true,
                                        v: true,
                                        link: false,
                                    });
                                    runner.block_on(docker.stop_container(&id, None)).expect
                                    ("Error removing old containers");
                                    runner.block_on(docker.remove_container(&id, options)).expect
                                    ("Error removing old containers");
                                }
                            }
                        }
                    }
                }
                let container = setup_container();
                Ok(SyncRunner::start(container).unwrap())
            }
        }.unwrap();
        spawn(move || {
            let rx = POSTGRES_CHANNEL.rx.lock().unwrap();
            while let Ok(command) = rx.recv() {
                match command {
                    ContainerCommands::Stop => {
                        running_container.rm().expect("TODO: panic message");
                        break;
                    }
                    ContainerCommands::Cleanup => {
                        clear_users();
                        clear_notifications();
                    }
                }
            }
        });
    }

    pub fn setup_container() -> ContainerRequest<Postgres> {
        Postgres::default().with_mapped_port(55002, ContainerPort::Tcp(5432))
    }

    fn clear_users() {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        diesel::delete(users)
            .execute(&mut crate::get_connection())
            .unwrap();
    }

    fn clear_notifications() {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::*;
        diesel::delete(notifications)
            .execute(&mut crate::get_connection())
            .unwrap();
    }

    pub fn create_random_user() -> User {
        User::new(
            0,
            "testuser",
            Role::User,
            Some(digest("testuser")),
            Utc::now().naive_utc(),
            false,
        )
    }
}
