use tokio::runtime::{Builder, Runtime};

#[cfg(feature = "multicore")]
pub(crate) fn build(mut cores: usize) -> Runtime {
    let cpus = num_cpus::get();
    debug_assert!(cpus > 0, "At least one CPU must be available");
    if cores > cpus {
        warn!(
            "cpus:{:?},
            HPX_PROXY_CORES = {:?} Ignoring configuration due to insufficient resources",
            cpus, cores
        );
        cores = cpus;
    }

    match cores {
        // `0` is unexpected, but it's a wild world out there.
        0 | 1 => {
            info!("Using single-threaded proxy runtime");
            Builder::new_current_thread()
                .enable_all()
                .thread_name("hpx-mesh")
                .build()
                .expect("failed to build basic runtime!")
        }
        num_cpus => {
            info!("cores:{:?} Using multi-threaded proxy runtime", cores);
            Builder::new_multi_thread()
                .enable_all()
                .thread_name("hpx-mesh")
                .worker_threads(num_cpus)
                .max_blocking_threads(num_cpus)
                .build()
                .expect("failed to build threaded runtime!")
        }
    }
}

#[cfg(not(feature = "multicore"))]
pub(crate) fn build() -> Runtime {
    Builder::new()
        .enable_all()
        .thread_name("proxy")
        .basic_scheduler()
        .build()
        .expect("failed to build basic runtime!")
}
