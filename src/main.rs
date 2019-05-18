extern crate hyper;
extern crate rifling;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::env;
use std::error;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use git2 as git;
use hyper::rt::{self, Future};
use rifling::{Constructor, Delivery, Hook};
use snafu::{ResultExt, Snafu};

lazy_static! {
    static ref PORT: u16 = env::var("PORT")
        .expect("Expected $PORT environment variable")
        .parse::<u16>()
        .expect("Expected $PORT to be integer");
    static ref SECRET: String = env::var("SECRET").expect("Expected $SECRET environment variable");
}

fn main() {
    env_logger::init();

    let addr = ([127, 0, 0, 1], *PORT).into();

    let mut handler = Constructor::new();
    let hook = Hook::new("*", Some(SECRET.to_string()), |delivery: &Delivery| {
        if let Some(payload) = &delivery.payload {
            if payload["repository"]["full_name"] == "peachcloud/peach-packages"
                && payload["ref"] == "refs/heads/release"
            {
                info!("Packages are released, running build process");
                if let Err(error) = run() {
                    error!("{}", error)
                }
                return;
            }
        }
        info!("Ignoring web hook");
    });
    handler.register(hook);

    let server = hyper::Server::bind(&addr)
        .serve(handler)
        .map_err(|e: hyper::Error| error!("Error: {:?}", e));

    info!("Server listening on {}", addr);

    rt::run(server);
}

fn run() -> Result<(), Box<dyn error::Error>> {
    setup_repo()?;
    build_packages()?;

    Ok(())
}

fn setup_repo() -> Result<(), git::Error> {
    let path = PathBuf::from("./packages");
    let repo = if path.exists() {
        info!("Opening repository at {}", path.display());
        git::Repository::open(path)
    } else {
        let url = "git://github.com/peachcloud/peach-packages";
        info!("Cloning repository from {} at {}", url, path.display());
        git::Repository::clone_recurse(url, path)
    }?;

    info!("git fetch origin release");
    let mut origin_remote = repo.find_remote("origin")?;
    origin_remote.fetch(&["release"], None, None)?;

    info!("git reset --hard origin/release");
    let oid = repo.refname_to_id("refs/remotes/origin/release")?;
    let object = repo.find_object(oid, None).unwrap();
    repo.reset(&object, git::ResetType::Hard, None)?;

    info!("git submodule update --init --recursive");
    repo.submodules()?
        .into_iter()
        .try_for_each(|mut submodule| submodule.update(true, None))?;

    Ok(())
}

fn build_packages() -> Result<(), Error> {
    info!("Building packages");
    let status = Command::new("./build.sh")
        .current_dir("./packages")
        .status()
        .context(CommandFailed {})?;

    if status.success() {
        Ok(())
    } else {
        BuildFailed {}.fail()
    }
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Failed to execute process: {}", source))]
    CommandFailed { source: std::io::Error },
    #[snafu(display("Build failed"))]
    BuildFailed,
}
