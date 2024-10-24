use nusb::Interface;
use nusb::transfer::{ControlOut, ControlType, Recipient};
use thiserror::Error;

pub const VENDOR_ID: u16 = 0x046d;
pub const PRODUCT_ID: u16 = 0x0a78;

pub fn detach_and_claim_interface() -> Result<Interface, ClaimInterfaceError> {
    let device = nusb::list_devices()?
        .find(|dev| dev.vendor_id() == VENDOR_ID && dev.product_id() == PRODUCT_ID)
            .ok_or(ClaimInterfaceError::InterfaceNotFound)?
        .open()?;

    Ok(device.detach_and_claim_interface(0x02)?)
}

#[derive(Error, Debug)]
pub enum ClaimInterfaceError {
    #[error("could not find \"g560\" device with vendor_id {0} and product_id {1}", VENDOR_ID, PRODUCT_ID)]
    InterfaceNotFound,
    #[error("could not open \"g560\" device")]
    InterfaceClaimFailed(#[from] nusb::Error),
}

pub async fn send_raw_command(
    interface: &Interface,
    value: &[u8; 10]
)
-> Result<(), SendCommandError>
{
    let prefix = [0x11, 0xff, 0x04, 0x3a];
    let suffix = [0x00; 6];

    let mut data = Vec::with_capacity(20);
    data.extend_from_slice(&prefix);
    data.extend_from_slice(value);
    data.extend_from_slice(&suffix);
    
    let control_out = ControlOut {
        recipient: Recipient::Interface,
        control_type: ControlType::Class,
        request: 0x09,
        value: 0x0211,
        index: 0x02,
        data: &data,
    };

    interface.control_out(control_out).await.into_result()?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum SendCommandError {
    #[error("control out failed")]
    ControlOutFailed(#[from] nusb::transfer::TransferError),
}
