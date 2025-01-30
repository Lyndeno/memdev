use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use udev::{Device, Entry};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub devices: Vec<MemDevice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemDevice {
    pub manufacturer: Option<String>,
    pub frequency: Option<u64>,
    pub form_factor: Option<String>,
    pub mem_type: Option<String>,
    pub extra_props: HashMap<String, String>,
}

impl MemDevice {
    /// Returns new `MemDevice`
    ///
    /// # Errors
    /// Returns error if getting memory information fails
    pub fn new(index: usize, memmap: &HashMap<String, String>) -> Self {
        let mut propmap = HashMap::new();
        for (key, value) in memmap.iter() {
            if let Some(name) = key.strip_prefix(&format!("MEMORY_DEVICE_{index}_")) {
                propmap.insert(name.to_string(), value.to_string());
            }
        }

        MemDevice::from(propmap)
    }
}

impl From<HashMap<String, String>> for MemDevice {
    fn from(mut extra_props: HashMap<String, String>) -> Self {
        //let mut extra_props = value.clone();
        let manufacturer = extra_props.remove("MANUFACTURER");
        let frequency = extra_props
            .remove("CONFIGURED_SPEED_MTS")
            .and_then(|x| str::parse::<u64>(&x).ok());
        let form_factor = extra_props.remove("FORM_FACTOR");
        let mem_type = extra_props.remove("TYPE");

        Self {
            manufacturer,
            frequency,
            form_factor,
            mem_type,
            extra_props,
        }
    }
}

impl Memory {
    /// Return a new memory object.
    /// # Errors
    ///
    /// Will return an error if the memory stats cannot be parsed.
    /// Does not error on failure to obtain smbios information
    pub fn new() -> Result<Self> {
        let udev = Device::from_syspath(Path::new("/sys/devices/virtual/dmi/id"))?;
        let props = udev.properties();
        let props_vec: Vec<Entry<'_>> = props.collect();

        let mut propmap = HashMap::new();

        for prop in props_vec {
            let k = prop.name().to_string_lossy().to_string();
            let v = prop.value().to_string_lossy().to_string();
            propmap.insert(k, v);
        }

        let count_entry = propmap
            .get("MEMORY_ARRAY_NUM_DEVICES")
            .ok_or(Error::Missing)?;

        let count = str::parse::<usize>(count_entry)?;

        let mut devs = Vec::with_capacity(count);

        for i in 0..count {
            devs.push(MemDevice::new(i, &propmap));
        }

        Ok(Self {
            // This will usually error do to permission errors, so just wrap it None instead
            // as it is not needed for basic use
            devices: devs,
        })
    }

    pub fn avg_frequency(&self) -> u64 {
        let mut v = Vec::new();
        for dev in &self.devices {
            if let Some(f) = dev.frequency {
                v.push(f);
            }
        }
        avg_frequency(v)
    }
}

fn sum_frequency(f: Vec<u64>) -> u64 {
    let mut sum = 0;
    for freq in f {
        sum += freq;
    }
    sum
}

fn avg_frequency(f: Vec<u64>) -> u64 {
    let count = f.len();
    if count > 0 {
        sum_frequency(f) / count as u64
    } else {
        0
    }
}
