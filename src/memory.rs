use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use udev::{Device, Entry};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub devices: Option<Vec<MemDevice>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemDevice {
    manufacturer: Option<String>,
    frequency: Option<u64>,
    form_factor: Option<String>,
    mem_type: Option<String>,
    extra_props: HashMap<String, String>,
}

impl MemDevice {
    /// Returns new `MemDevice`
    ///
    /// # Errors
    /// Returns error if getting memory information fails
    pub fn new(index: usize) -> Result<Self> {
        let udev = Device::from_syspath(Path::new("/sys/devices/virtual/dmi/id"))?;
        let props = udev.properties();
        let props_vec: Vec<Entry<'_>> = props.collect();

        let mut propmap = HashMap::new();

        for prop in &props_vec {
            if let Some(clean_name) = prop
                .name()
                .to_string_lossy()
                .to_string()
                .strip_prefix(&format!("MEMORY_DEVICE_{index}_"))
            {
                propmap.insert(
                    clean_name.to_string(),
                    prop.value().to_string_lossy().into_owned(),
                );
            }
        }

        Ok(MemDevice::from(propmap))
    }

    fn pull_value<T: FromStr>(&self, name: &str) -> Option<T> {
        if let Some(v) = self.extra_props.get(name) {
            return str::parse::<T>(v).ok();
        }
        None
    }
}

impl From<HashMap<String, String>> for MemDevice {
    fn from(mut extra_props: HashMap<String, String>) -> Self {
        //let mut extra_props = value.clone();
        let manufacturer = extra_props.remove("MANUFACTURER");
        let frequency = extra_props
            .remove("CONFIGURED_SPEED_MTS")
            .map(|x| str::parse::<u64>(&x).ok())
            .flatten();
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
        let mut props = udev.properties();

        let count_entry = props
            .find(|x| {
                x.name()
                    .to_string_lossy()
                    .contains("MEMORY_ARRAY_NUM_DEVICE")
            })
            .ok_or(Error::Missing)?;

        let count = str::parse::<usize>(&count_entry.value().to_string_lossy())?;

        let mut devs = Vec::with_capacity(count);

        for i in 0..count {
            devs.push(MemDevice::new(i)?);
        }

        Ok(Self {
            // This will usually error do to permission errors, so just wrap it None instead
            // as it is not needed for basic use
            devices: Some(devs),
        })
    }

    fn get_type(&self) -> Vec<String> {
        let mut memtype = Vec::new();
        if let Some(v) = &self.devices {
            for dev in v {
                if let Some(x) = dev.mem_type.clone() {
                    memtype.push(x);
                }
            }
        }

        let mut string_vec: Vec<String> = memtype
            .iter()
            .map(std::string::ToString::to_string)
            .collect();

        let set: HashSet<_> = string_vec.drain(..).collect();
        string_vec.extend(set);

        string_vec
    }

    fn get_formfactor(&self) -> Vec<String> {
        let mut memff = Vec::new();
        if let Some(v) = &self.devices {
            for dev in v {
                if let Some(x) = dev.form_factor.clone() {
                    memff.push(x);
                }
            }
        }

        let mut string_vec: Vec<String> =
            memff.iter().map(std::string::ToString::to_string).collect();

        let set: HashSet<_> = string_vec.drain(..).collect();
        string_vec.extend(set);

        string_vec
    }

    fn get_speed(&self) -> Vec<u64> {
        let mut speeds = Vec::new();
        if let Some(v) = &self.devices {
            for dev in v {
                if let Some(x) = dev.frequency {
                    speeds.push(x);
                }
            }
        }
        speeds
    }

    fn display_unit(&self, used: f64, total: f64, unit: &str) -> String {
        let typestring = print_strings(self.get_type());
        let avg_freq = avg_frequency(self.get_speed());
        let formfactors = print_strings(self.get_formfactor());

        let mut s = String::new();
        s.push_str(&display_mem_unit(used, total, unit));

        if let Some(v) = typestring {
            s.push(' ');
            s.push_str(&v);
        }

        if let Some(v) = formfactors {
            s.push_str(&format!(" ({v})"));
        }

        if avg_freq > 0 {
            s.push_str(&format!(" @ {avg_freq} MHz"));
        }
        s
    }
}

fn display_mem_unit(used: f64, total: f64, unit: &str) -> String {
    format!("{used:.2}{unit} / {total:.2}{unit}")
}

fn print_strings(strings: Vec<String>) -> Option<String> {
    if strings.is_empty() {
        None
    } else {
        let mut list = String::new();

        let mut typeiter = strings.into_iter();

        if let Some(x) = typeiter.next() {
            list.push_str(&x);
            for y in typeiter {
                list.push_str(", ");
                list.push_str(&y);
            }
        }
        Some(list)
    }
}

fn sum_frequency(f: Vec<u64>) -> u64 {
    let mut sum = 0;
    for freq in f {
        sum += freq;
    }
    sum
}

#[allow(clippy::cast_precision_loss)]
fn avg_frequency(f: Vec<u64>) -> u64 {
    let count = f.len();
    if count > 0 {
        sum_frequency(f) / count as u64
    } else {
        0
    }
}
