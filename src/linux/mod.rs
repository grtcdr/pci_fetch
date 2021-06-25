use crate::classes::*;
use crate::extra::*;
use crate::traits::*;
use std::path::PathBuf;

/// This is where PCI devices are located.
const PATH_TO_PCI_DEVICES: &str = "/sys/bus/pci/devices";
/// This is where the pci.ids file is located.
const PATH_TO_PCI_IDS: &str = "/usr/share/hwdata/pci.ids";

#[derive(Debug)]
pub struct LinuxPCIDevice {
    path: PathBuf,
    address: String,
    class_id: String,
    class_name: String,
    vendor_id: String,
    vendor_name: String,
    device_id: String,
    device_name: String,
    numa_node: isize,
}

impl Properties for LinuxPCIDevice {
    fn new(path: &str) -> Self {
        let mut device = LinuxPCIDevice {
            path: PathBuf::from(path),
            address: String::new(),
            class_id: String::new(),
            class_name: String::new(),
            vendor_id: String::new(),
            vendor_name: String::new(),
            device_id: String::new(),
            device_name: String::new(),
            numa_node: -1,
        };

        Self::init(&mut device);

        if let Ok(lines) = read_lines(PATH_TO_PCI_IDS) {
            for line in lines {
                if let Ok(l) = line {
                    // Ignore empty lines, comments, and class definitions
                    if l.len() == 0 || l.starts_with("#") || l.starts_with("C") {
                        continue;
                    }

                    if !l.starts_with("\t") {
                        // This is the condition for vendors
                        let vendor_id = l[..4].trim_start();
                        let vendor_name = l[4..].trim_start();

                        if device.vendor_id() == vendor_id {
                            device.vendor_name = vendor_name.to_string();
                        }
                    } else if l.starts_with("\t") {
                        // This is the condition for devices
                        let device_id = l[..5].trim_start();
                        let device_name = l[5..].trim_start();

                        if device.device_id() == device_id {
                            device.device_name = device_name.to_string();
                        }
                    }
                }
            }
        }

        return device;
    }

    fn init(&mut self) {
        self.set_address();
        self.set_class_id();
        self.set_vendor_id();
        self.set_device_id();
        self.set_numa_node();
        self.set_class_name();
    }

    fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    fn address(&self) -> String {
        self.address.to_owned()
    }

    fn class_id(&self) -> String {
        self.class_id.to_owned()
    }

    fn vendor_id(&self) -> String {
        self.vendor_id.to_owned()
    }

    fn device_id(&self) -> String {
        self.device_id.to_owned()
    }

    fn numa_node(&self) -> isize {
        self.numa_node.to_owned()
    }

    fn class_name(&self) -> String {
        self.class_name.to_owned()
    }

    fn vendor_name(&self) -> String {
        self.vendor_name.to_owned()
    }

    fn device_name(&self) -> String {
        self.device_name.to_owned()
    }

    fn set_path(&mut self, p: PathBuf) {
        self.path = p;
    }

    fn set_address(&mut self) {
        self.address = basename(
            self.path()
                .as_path()
                .display()
                .to_string()
                .replace("0000:", ""),
        );
    }

    fn set_class_id(&mut self) {
        if let Ok(mut str) = std::fs::read_to_string(&self.path().join("class")) {
            str.pop();
            str = str.trim_start_matches("0x").chars().take(2).collect();
            self.class_id = str;
        }
    }

    fn set_vendor_id(&mut self) {
        if let Ok(mut str) = std::fs::read_to_string(&self.path().join("vendor")) {
            str = str.trim_start_matches("0x").to_string();
            str.pop();
            self.vendor_id = str;
        }
    }

    fn set_device_id(&mut self) {
        if let Ok(mut str) = std::fs::read_to_string(&self.path().join("device")) {
            str = str.trim_start_matches("0x").to_string();
            str.pop();
            self.device_id = str;
        }
    }

    fn set_numa_node(&mut self) {
        if let Ok(v) = std::fs::read_to_string(&self.path().join("numa_node")) {
            if let Ok(p) = v.parse::<isize>() {
                return self.numa_node = p;
            }
        }
    }

    /// This function sets the PCI device's `class_name` associated
    /// with its `class_id` *as defined by **pci.ids***.
    fn set_class_name(&mut self) {
        if !&self.class_id.is_empty() {
            self.class_name = match &self.class_id[..] {
                "00" => DeviceClass::Unclassified.to_string(),
                "01" => DeviceClass::MassStorageController.to_string(),
                "02" => DeviceClass::NetworkController.to_string(),
                "03" => DeviceClass::DisplayController.to_string(),
                "04" => DeviceClass::MultimediaController.to_string(),
                "05" => DeviceClass::MemoryController.to_string(),
                "06" => DeviceClass::PCIBridge.to_string(),
                "07" => DeviceClass::CommunicationController.to_string(),
                "08" => DeviceClass::GenericSystemPeripheral.to_string(),
                "09" => DeviceClass::InputDeviceController.to_string(),
                "0a" => DeviceClass::DockingStation.to_string(),
                "0b" => DeviceClass::Processor.to_string(),
                "0c" => DeviceClass::SerialBusController.to_string(),
                "0d" => DeviceClass::WirelessController.to_string(),
                "0e" => DeviceClass::IntelligentController.to_string(),
                "0f" => DeviceClass::SatelliteCommunicationsController.to_string(),
                "10" => DeviceClass::EncryptionController.to_string(),
                "11" => DeviceClass::SignalProcessingController.to_string(),
                "12" => DeviceClass::ProcessingAccelerators.to_string(),
                "13" => DeviceClass::NonEssentialInstrumentation.to_string(),
                _ => DeviceClass::Unknown.to_string(),
            }
        }
    }

    fn set_vendor_name(&mut self, name: String) {
        self.vendor_name = name;
    }

    fn set_device_name(&mut self, name: String) {
        self.device_name = name;
    }
}

impl std::fmt::Display for LinuxPCIDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl Fetch for LinuxPCIDevice {
    fn fetch() -> Vec<LinuxPCIDevice> {
        let mut devices = Vec::new();
        let dir_entries = list_dir_entries(PATH_TO_PCI_DEVICES);
        for dir in dir_entries {
            let device = LinuxPCIDevice::new(dir.to_str().unwrap());
            devices.push(device);
        }
        return devices;
    }

    fn fetch_by_class(class: DeviceClass) -> Vec<LinuxPCIDevice> {
        let mut devices = Vec::new();
        let dir_entries = list_dir_entries(PATH_TO_PCI_DEVICES);
        for dir in dir_entries {
            let device: LinuxPCIDevice = LinuxPCIDevice::new(dir.to_str().unwrap());
            if device.class_name() == class.to_string() {
                devices.push(device);
            }
        }

        return devices;
    }

    fn fetch_gpus() -> Vec<LinuxPCIDevice> {
        let mut gpus: Vec<LinuxPCIDevice> = Self::fetch_by_class(DeviceClass::DisplayController);
        for gpu in &mut gpus {
            let whole_name = gpu.device_name();
            if let Some(start_bytes) = whole_name.find("[") {
                let end_bytes = whole_name.find("]").unwrap_or(whole_name.len());
                let new_name = &whole_name[start_bytes + 1..end_bytes];
                gpu.set_device_name(new_name.to_string());
            }
        }
        gpus
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_address() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.address(), "");
    }

    #[test]
    fn test_path() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.path(), PathBuf::new());
    }

    #[test]
    fn test_class_name() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.class_name(), "");
    }

    #[test]
    fn test_class_id() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.device_id(), "");
    }

    #[test]
    fn test_vendor_name() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.vendor_name(), "");
    }

    #[test]
    fn test_vendor_id() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.vendor_id(), "");
    }

    #[test]
    fn test_device_name() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.vendor_name(), "");
    }

    #[test]
    fn test_device_id() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.device_id(), "");
    }

    #[test]
    fn test_numa_node() {
        let device = LinuxPCIDevice::new("/sys/bus/pci/devices/0000:00:00.0");
        assert_ne!(device.numa_node().to_string(), "");
    }
}
