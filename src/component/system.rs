pub(crate) struct SystemArgs {
    pub(crate) os: bool,
    pub(crate) mem: bool,
    pub(crate) cpu: bool,
}

pub(crate) struct System<'a> {
    os_name: Option<String>,
    os_version: Option<String>,
    os_kernel_version: Option<String>,
    os_distribution_id: Option<String>,

    mem_total_bytes: Option<u64>,
    mem_used_bytes: Option<u64>,
    mem_swap_total_bytes: Option<u64>,
    mem_swap_used_bytes: Option<u64>,

    cpu_arch: Option<String>,
    cpu: Option<Vec<Cpu<'a>>>,
}

impl<'a> System<'a> {
    pub(crate) fn new(args: SystemArgs, system: &'a sysinfo::System) -> Self {
        Self {
            os_name: if args.os {
                sysinfo::System::name()
            } else {
                None
            },

            os_version: if args.os {
                sysinfo::System::os_version()
            } else {
                None
            },

            os_kernel_version: if args.os {
                sysinfo::System::kernel_version()
            } else {
                None
            },

            os_distribution_id: if args.os {
                Some(sysinfo::System::distribution_id())
            } else {
                None
            },

            mem_total_bytes: if args.mem {
                Some(system.total_memory())
            } else {
                None
            },

            mem_used_bytes: if args.mem {
                Some(system.used_memory())
            } else {
                None
            },

            mem_swap_total_bytes: if args.mem {
                Some(system.total_swap())
            } else {
                None
            },

            mem_swap_used_bytes: if args.mem {
                Some(system.used_swap())
            } else {
                None
            },

            cpu_arch: if args.cpu {
                sysinfo::System::cpu_arch()
            } else {
                None
            },

            cpu: if args.cpu { Cpu::new(system) } else { None },
        }
    }
}

impl<'a> std::fmt::Debug for System<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("System");

        if let Some(ref os_name) = self.os_name {
            s.field("os_name", os_name);
        }

        if let Some(ref os_version) = self.os_version {
            s.field("os_version", os_version);
        }

        if let Some(ref os_kernel_version) = self.os_kernel_version {
            s.field("os_kernel_version", os_kernel_version);
        }

        if let Some(ref os_distribution_id) = self.os_distribution_id {
            s.field("os_distribution_id", os_distribution_id);
        }

        if let Some(mem_total_bytes) = self.mem_total_bytes {
            s.field("mem_total_bytes", &mem_total_bytes);
        }

        if let Some(mem_used_bytes) = self.mem_used_bytes {
            s.field("mem_used_bytes", &mem_used_bytes);
        }

        if let Some(mem_swap_total_bytes) = self.mem_swap_total_bytes {
            s.field("mem_swap_total_bytes", &mem_swap_total_bytes);
        }

        if let Some(mem_swap_used_bytes) = self.mem_swap_used_bytes {
            s.field("mem_swap_used_bytes", &mem_swap_used_bytes);
        }

        if let Some(ref cpu_arch) = self.cpu_arch {
            s.field("cpu_arch", cpu_arch);
        }

        if let Some(ref cpu) = self.cpu {
            s.field("cpu", cpu);
        }

        s.finish()
    }
}

pub(crate) struct Cpu<'a> {
    name: &'a str,
    model: &'a str,
    vendor_id: &'a str,
}

impl<'a> Cpu<'a> {
    pub(crate) fn new(system: &'a sysinfo::System) -> Option<Vec<Cpu<'a>>> {
        let cpus = system.cpus();

        if cpus.is_empty() {
            None
        } else {
            Some(CpuSlice(cpus).into())
        }
    }
}

impl<'a> std::fmt::Debug for Cpu<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("name", &self.name)
            .field("model", &self.model.trim())
            .field("vendor_id", &self.vendor_id)
            .finish()
    }
}

pub(crate) struct CpuSlice<'a>(&'a [sysinfo::Cpu]);

impl<'a> From<CpuSlice<'a>> for Vec<Cpu<'a>> {
    fn from(slice: CpuSlice<'a>) -> Self {
        slice
            .0
            .iter()
            .map(|cpu| Cpu {
                name: cpu.name().trim(),
                model: cpu.brand(),
                vendor_id: cpu.vendor_id(),
            })
            .collect()
    }
}
