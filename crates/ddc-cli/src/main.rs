//! Composition root of `ddc-cli`: parses the command line, builds the
//! backend — the fixed fake behind `--fake`, else the real DDC/CI backend
//! with its on-disk capabilities cache — and runs the command.

#![forbid(unsafe_code)]

use std::io;
use std::process::ExitCode;

use clap::Parser;
use ddc_adapters::{
    CachingMonitorBackend, DdcHiMonitorBackend, InMemoryMonitorBackend, default_cache_dir,
};
use ddc_cli::fixture::fixture_monitor;
use ddc_cli::{Cli, report_startup_failure, run};
use ddc_core::app::SoftwareOsd;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = io::stdout().lock();
    let mut err = io::stderr().lock();
    let exit = if cli.fake {
        let fake = InMemoryMonitorBackend::builder()
            .monitor(fixture_monitor())
            .build();
        run(&cli, &SoftwareOsd::new(fake), &|_| {}, &mut out, &mut err)
    } else {
        match (DdcHiMonitorBackend::new(), default_cache_dir()) {
            (Err(error), _) => report_startup_failure(&cli, error, &mut err),
            (Ok(real), None) => run(&cli, &SoftwareOsd::new(real), &|_| {}, &mut out, &mut err),
            (Ok(real), Some(dir)) => {
                let cached = CachingMonitorBackend::new(real, dir);
                let refresh = |id: &_| cached.invalidate(id);
                run(
                    &cli,
                    &SoftwareOsd::new(&cached),
                    &refresh,
                    &mut out,
                    &mut err,
                )
            }
        }
    };
    exit.into()
}
