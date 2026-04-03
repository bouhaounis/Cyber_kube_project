use crate::{api::event_stream::EventStreamHub, k8s_metadata::KubernetesMetadataCache};

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use crate::ebpf::{events, loader::EbpfLoader};
    use anyhow::{Context, Result};
    use aya::{maps::perf::AsyncPerfEventArray, util::online_cpus, Bpf};
    use bytes::BytesMut;
    use tokio::task::JoinHandle;
    use tracing::{error, warn};

    pub struct LoadedProgram {
        pub name: String,
        pub bpf: Bpf,
        reader_tasks: Vec<JoinHandle<()>>,
    }

    pub struct EbpfManager {
        programs: Vec<LoadedProgram>,
        event_hub: EventStreamHub,
        metadata_cache: KubernetesMetadataCache,
    }

    impl EbpfManager {
        pub fn new(event_hub: EventStreamHub, metadata_cache: KubernetesMetadataCache) -> Self {
            Self {
                programs: Vec::new(),
                event_hub,
                metadata_cache,
            }
        }

        pub fn load_program(&mut self, path: &str) -> Result<()> {
            let mut loader = EbpfLoader::load(path)?;
            let reader_tasks = self.spawn_event_reader(loader.bpf_mut())?;
            self.programs.push(LoadedProgram {
                name: path.to_string(),
                bpf: loader.into_bpf(),
                reader_tasks,
            });
            Ok(())
        }

        pub fn unload_program(&mut self, name: &str) -> Result<()> {
            if let Some(index) = self
                .programs
                .iter()
                .position(|program| program.name.contains(name))
            {
                let program = self.programs.remove(index);
                for task in program.reader_tasks {
                    task.abort();
                }
            }
            Ok(())
        }

        pub fn list_programs(&self) -> Vec<&str> {
            self.programs
                .iter()
                .map(|program| {
                    let _ = &program.bpf;
                    program.name.as_str()
                })
                .collect()
        }

        fn spawn_event_reader(&self, bpf: &mut Bpf) -> Result<Vec<JoinHandle<()>>> {
            let map = bpf
                .take_map("SECURITY_EVENTS")
                .context("SECURITY_EVENTS map not found in loaded eBPF object")?;
            let mut perf_array = AsyncPerfEventArray::try_from(map)?;
            let mut tasks = Vec::new();

            for cpu_id in online_cpus().context("enumerating online CPUs for eBPF event reader")? {
                let mut buffer = perf_array.open(cpu_id, None)?;
                let event_hub = self.event_hub.clone();
                let metadata_cache = self.metadata_cache.clone();

                tasks.push(tokio::spawn(async move {
                    let mut buffers = (0..16)
                        .map(|_| BytesMut::with_capacity(1024))
                        .collect::<Vec<_>>();

                    loop {
                        match buffer.read_events(&mut buffers).await {
                            Ok(events_read) => {
                                for raw in buffers.iter().take(events_read.read) {
                                    match events::process(raw.as_ref(), &metadata_cache) {
                                        Ok(event) => match event.try_into_proto_event() {
                                            Ok(proto_event) => event_hub.publish(proto_event),
                                            Err(err) => warn!(
                                                "failed to convert eBPF event into gRPC payload: {err:#}"
                                            ),
                                        },
                                        Err(err) => {
                                            warn!("failed to parse eBPF event: {err:#}");
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                error!("failed reading eBPF events from CPU {cpu_id}: {err:#}");
                                break;
                            }
                        }
                    }
                }));
            }

            Ok(tasks)
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::*;
    use crate::ebpf::loader::EbpfLoader;
    use anyhow::Result;

    pub struct LoadedProgram {
        pub name: String,
    }

    pub struct EbpfManager {
        programs: Vec<LoadedProgram>,
        event_hub: EventStreamHub,
        metadata_cache: KubernetesMetadataCache,
    }

    impl EbpfManager {
        pub fn new(event_hub: EventStreamHub, metadata_cache: KubernetesMetadataCache) -> Self {
            Self {
                programs: Vec::new(),
                event_hub,
                metadata_cache,
            }
        }

        pub fn load_program(&mut self, path: &str) -> Result<()> {
            // Non-Linux builds keep the same API surface so the host workspace checks cleanly,
            // but still return a real runtime error if someone tries to enable eBPF here.
            let _ = &self.event_hub;
            let _ = &self.metadata_cache;
            let _ = EbpfLoader::load(path)?;
            self.programs.push(LoadedProgram {
                name: path.to_string(),
            });
            Ok(())
        }

        pub fn unload_program(&mut self, name: &str) -> Result<()> {
            self.programs.retain(|program| !program.name.contains(name));
            Ok(())
        }

        pub fn list_programs(&self) -> Vec<&str> {
            self.programs
                .iter()
                .map(|program| program.name.as_str())
                .collect()
        }
    }
}

pub use imp::{EbpfManager, LoadedProgram};
