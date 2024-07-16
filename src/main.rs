use rusb::{
    Context,
    ConfigDescriptor,
    Device, 
    DeviceDescriptor, 
    DeviceHandle, 
    DeviceList, 
    Direction, 
    EndpointDescriptor, 
    GlobalContext, 
    InterfaceDescriptor, 
    Language, 
    Result, 
    Speed, 
    TransferType, 
    UsbContext
};

use std::{
    str,
    iter::Product, 
    time::Duration
};

use usb_ids::{self, FromId};

use clap::Parser;

#[derive(Debug)]
struct UsbDevice<T: UsbContext> {
    handle: DeviceHandle<T>,
    language: Language,
    timeout: Duration,
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// command to send
    #[arg(short, long)]
    command: String,
}

fn main() {
    //let args = Args::parse();
    //let cmds: Vec<&str> = args.command.split_whitespace().collect();
    let vid = 0x06cb;
    let pid = 0x000f;
    //rusb::set_log_level(rusb::LogLevel::Debug);
    let mut context = Context::new().unwrap();
    let (mut device, device_desc, mut device_handle) = open_device(&mut context, vid, pid).unwrap();
    let out_ep = find_writable_endpoint(&mut device,
                                        &device_desc,
                                        TransferType::Bulk).unwrap();
    let in_ep = find_readable_endpoint(&mut device,
                                       &device_desc,
                                       TransferType::Bulk).unwrap();
    configure_endpoint(&mut device_handle, &out_ep).unwrap();
    configure_endpoint(&mut device_handle, &in_ep).unwrap();
    let commands = ["identify\n", "target=0 power on vdd=1800 vpu=3300 vled=1200 vddtx=1800\n",
        "target=0 config raw pl=i2c pull-ups=yes attnPull-ups=yes speed=400 attn=none base64=no\n",
        "target=0 raw bus-addr=40 wr=0200005a\n",
        "target=0 raw bus-addr=40 rd=4\n",
        "target=0 raw bus-addr=40 rd=38\n",
        "target=0 raw bus-addr=40 wr=0a000017\n",
        "target=0 raw bus-addr=40 rd=4\n",
        "target=0 raw bus-addr=40 rd=42\n",];
    /*
    for cmd in cmds {
        if cmd.len() > 0 {
            println!("{}", cmd);
        }

        write_endpoint(&mut device_handle, &out_ep, TransferType::Bulk, format!("{}\n", cmd).as_str());
        let mut buf = Vec::new();
        read_endpoint(&mut device_handle, &in_ep, TransferType::Bulk, &mut buf);
        if buf.len() > 0 {
            println!("{}", str::from_utf8(&buf).unwrap());
        }
    }
    */

    for c in commands {
        if c.len() > 0 {
            println!("{}", c);
        }

        write_endpoint(&mut device_handle, &out_ep, TransferType::Bulk, c);
        let mut buf = Vec::new();
        read_endpoint(&mut device_handle, &in_ep, TransferType::Bulk, &mut buf);
        if buf.len() > 0 {
            println!("{}", str::from_utf8(&buf).unwrap());
        }
    }


}

fn open_device<T: UsbContext>(context: &mut T,
                              vid: u16,
                              pid: u16,) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };
    
    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        
        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => panic!("Device found but failed to open: {}", e),
            }
        }
    }
    None
}

/**
fn list_devices() -> Result<()> {
    let timeout = Duration::from_secs(1);
    for mut device in DeviceList::new()?.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        let syna_id = device_desc.vendor_id();
        let product_id = device_desc.product_id();
        if syna_id == 0x06cb && product_id == 0xf {
        let mut usb_device = device.open().unwrap();
        
        
        
            println!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x} {}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id(),
                get_speed(device.speed())
            );
            //print_device(&device_desc, &mut usb_device);

            for n in 0..device_desc.num_configurations() {
                let config_desc = match device.config_descriptor(n) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // print_config(&config_desc, &mut usb_device);

                for interface in config_desc.interfaces() {
                    for interface_desc in interface.descriptors() {
                        //print_interface(&interface_desc, &mut usb_device);

                        for endpoint_desc in interface_desc.endpoint_descriptors() {
                            //print_endpoint(&endpoint_desc);
                            let wr_ep = find_writable_endpoint(&mut device, 
                                &device_desc, 
                                TransferType::Bulk).unwrap();

                            let rd_ep = find_readable_endpoint(&mut device,
                                &device_desc, 
                                TransferType::Bulk).unwrap();
                            write_endpoint(&mut usb_device,
                                &wr_ep, TransferType::Bulk, "identify\n");
                            let mut buf = Vec::new();
                            read_endpoint(&mut usb_device,
                                &rd_ep,
                                TransferType::Bulk, &mut buf);
                            println!("buf = {:?}", str::from_utf8(&buf).unwrap());
                        }
                    }
                }
            }
        }
        
    }

    Ok(())
}

fn print_device<T: UsbContext>(device_desc: &DeviceDescriptor, handle: &mut Option<UsbDevice<T>>) {
    let vid = device_desc.vendor_id();
    let pid = device_desc.product_id();

    let vendor_name = match usb_ids::Vendor::from_id(device_desc.vendor_id()) {
        Some(vendor) => vendor.name(),
        None => "Unknown vendor",
    };

    let product_name =
        match usb_ids::Device::from_vid_pid(device_desc.vendor_id(), device_desc.product_id()) {
            Some(product) => product.name(),
            None => "Unknown product",
        };

    println!("Device Descriptor:");
    println!("  bLength              {:3}", device_desc.length());
    println!("  bDescriptorType      {:3}", device_desc.descriptor_type());
    println!(
        "  bcdUSB             {:2}.{}{}",
        device_desc.usb_version().major(),
        device_desc.usb_version().minor(),
        device_desc.usb_version().sub_minor()
    );
    println!("  bDeviceClass        {:#04x}", device_desc.class_code());
    println!(
        "  bDeviceSubClass     {:#04x}",
        device_desc.sub_class_code()
    );
    println!("  bDeviceProtocol     {:#04x}", device_desc.protocol_code());
    println!("  bMaxPacketSize0      {:3}", device_desc.max_packet_size());
    println!("  idVendor          {vid:#06x} {vendor_name}",);
    println!("  idProduct         {pid:#06x} {product_name}",);
    println!(
        "  bcdDevice          {:2}.{}{}",
        device_desc.device_version().major(),
        device_desc.device_version().minor(),
        device_desc.device_version().sub_minor()
    );
    println!(
        "  iManufacturer        {:3} {}",
        device_desc.manufacturer_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_manufacturer_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    println!(
        "  iProduct             {:3} {}",
        device_desc.product_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_product_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    println!(
        "  iSerialNumber        {:3} {}",
        device_desc.serial_number_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_serial_number_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    println!(
        "  bNumConfigurations   {:3}",
        device_desc.num_configurations()
    );
}


fn print_config<T: UsbContext>(config_desc: &ConfigDescriptor, handle: &mut Option<UsbDevice<T>>) {
    println!("  Config Descriptor:");
    println!("    bLength              {:3}", config_desc.length());
    println!(
        "    bDescriptorType      {:3}",
        config_desc.descriptor_type()
    );
    println!("    wTotalLength      {:#06x}", config_desc.total_length());
    println!(
        "    bNumInterfaces       {:3}",
        config_desc.num_interfaces()
    );
    println!("    bConfigurationValue  {:3}", config_desc.number());
    println!(
        "    iConfiguration       {:3} {}",
        config_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_configuration_string(h.language, config_desc, h.timeout)
            .unwrap_or_default())
    );
    println!("    bmAttributes:");
    println!("      Self Powered     {:>5}", config_desc.self_powered());
    println!("      Remote Wakeup    {:>5}", config_desc.remote_wakeup());
    println!("    bMaxPower           {:4}mW", config_desc.max_power());

    if !config_desc.extra().is_empty() {
        println!("    {:?}", config_desc.extra());
    } else {
        println!("    no extra data");
    }
}

fn print_interface<T: UsbContext>(
    interface_desc: &InterfaceDescriptor,
    handle: &mut Option<UsbDevice<T>>,
) {
    println!("    Interface Descriptor:");
    println!("      bLength              {:3}", interface_desc.length());
    println!(
        "      bDescriptorType      {:3}",
        interface_desc.descriptor_type()
    );
    println!(
        "      bInterfaceNumber     {:3}",
        interface_desc.interface_number()
    );
    println!(
        "      bAlternateSetting    {:3}",
        interface_desc.setting_number()
    );
    println!(
        "      bNumEndpoints        {:3}",
        interface_desc.num_endpoints()
    );
    println!(
        "      bInterfaceClass     {:#04x}",
        interface_desc.class_code()
    );
    println!(
        "      bInterfaceSubClass  {:#04x}",
        interface_desc.sub_class_code()
    );
    println!(
        "      bInterfaceProtocol  {:#04x}",
        interface_desc.protocol_code()
    );
    println!(
        "      iInterface           {:3} {}",
        interface_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_interface_string(h.language, interface_desc, h.timeout)
            .unwrap_or_default())
    );

    if interface_desc.extra().is_empty() {
        println!("    {:?}", interface_desc.extra());
    } else {
        println!("    no extra data");
    }
}

fn print_endpoint(endpoint_desc: &EndpointDescriptor) {
    println!("      Endpoint Descriptor:");
    println!("        bLength              {:3}", endpoint_desc.length());
    println!(
        "        bDescriptorType      {:3}",
        endpoint_desc.descriptor_type()
    );
    println!(
        "        bEndpointAddress    {:#04x} EP {} {:?}",
        endpoint_desc.address(),
        endpoint_desc.number(),
        endpoint_desc.direction()
    );
    println!("        bmAttributes:");
    println!(
        "          Transfer Type          {:?}",
        endpoint_desc.transfer_type()
    );
    println!(
        "          Synch Type             {:?}",
        endpoint_desc.sync_type()
    );
    println!(
        "          Usage Type             {:?}",
        endpoint_desc.usage_type()
    );
    println!(
        "        wMaxPacketSize    {:#06x}",
        endpoint_desc.max_packet_size()
    );
    println!(
        "        bInterval            {:3}",
        endpoint_desc.interval()
    );
}

fn get_speed(speed: Speed) -> &'static str {
    match speed {
        Speed::SuperPlus => "10000 Mbps",
        Speed::Super => "5000 Mbps",
        Speed::High => " 480 Mbps",
        Speed::Full => "  12 Mbps",
        Speed::Low => " 1.5 Mbps",
        _ => "(unknown)",
    }
}
**/

fn find_writable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,) -> Option<Endpoint> {
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };
    
            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == Direction::Out
                            && endpoint_desc.transfer_type() == transfer_type
                        {
                            return Some(Endpoint {
                                config: config_desc.number(),
                                iface: interface_desc.interface_number(),
                                setting: interface_desc.setting_number(),
                                address: endpoint_desc.address(),
                            });
                        }
                    }
                }
            }
        }
        None
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        let max_packet_size = endpoint_desc.max_packet_size();
                        //println!("max packet size of bulk in endpoint = {}", max_packet_size);
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

fn write_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
    transfer_type: TransferType,
    command: &str) {

    let buf = command.as_bytes();
    let timeout = Duration::from_secs(2);

    match transfer_type {
        TransferType::Interrupt => println!("No write to interrupt endpoint"),
        TransferType::Bulk => match handle.write_bulk(endpoint.address, &buf, timeout) {
            Ok(len) => {}
            Err(err) => println!("could not write to endpoint: {}", err)
        }
        _ => {}
    }
}

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
    transfer_type: TransferType,
    buf: &mut Vec<u8>,
) {
    let timeout = Duration::from_millis(20);
    let mut temp_buf = [0; 64];
    match transfer_type {
        TransferType::Interrupt => {
            match handle.read_interrupt(endpoint.address, &mut temp_buf, timeout) {
                Ok(len) => {
                    println!(" - read: {:?}", &buf[..len]);
                }
                Err(err) => println!("could not read from endpoint: {}", err)
            }
        }

        TransferType::Bulk => {
            loop {
                match handle.read_bulk(endpoint.address, &mut temp_buf, timeout) {
                    Ok(len) => {
                        buf.extend_from_slice(&temp_buf[..len]);
                        continue;
                    }
                    Err(err) => {
                        match err {
                            rusb::Error::Timeout => {}
                            _ => println!("Read bulk in endpoint err = {:?}", err)
                        }
                        break;
                    }
                }
            }
        }

        _ => {}
    }
}