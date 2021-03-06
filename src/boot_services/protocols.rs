use core::{
    mem,
    slice,
};
use types::{
    EfiBs,
    Handle,
    MemoryType,
    Status,
};
use super::{
    BootServices,
    Pool,
};


/// Globally-unique identifier, used in UEFI to distinguish protocols
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Guid {
    pub data_1: u32,
    pub data_2: u16,
    pub data_3: u16,
    pub data_4: [u8; 8],
}


bitflags! {
    /// Mode in which to open the protocol interface
    pub struct OpenProtocolAttributes: u32 {
        const BY_HANDLE_PROTOCOL = 0x0000_0001;
        const GET_PROTOCOL = 0x0000_0002;
        const TEST_PROTOCOL = 0x0000_0004;
        const BY_CHILD_CONTROLLER = 0x0000_0008;
        const BY_DRIVER = 0x0000_0010;
        const EXCLUSIVE = 0x0000_0020;
    }
}


/// Common functionality implemented by all protocols
pub trait Protocol {

    /// Returns the Guid that identifies this protocol
    fn guid() -> &'static Guid;
}


/// Specifies criteria used to search for available Handles
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum SearchType {
    AllHandles,
    ByRegisterNotify,
    ByProtocol,
}


impl BootServices {

    /// Returns a slice of handles that support the specified protocols
    pub fn locate_handle<'a>(
        &'a self,
        search_type: SearchType,
        protocol: Option<&Guid>,
        search_key: Option<*const ()>)
    -> Result<Pool<'a, [Handle]>, Status> {

        // Prepare arguments
        let protocol: *const Guid = protocol
            .map_or(0 as _, |g| g as _);
        let search_key = search_key
            .unwrap_or(0 as *const ());
        let mut buf_size = 0;
        let buf = 0 as *mut Handle;

        // Make an initial call to determine the required buffer size
        let res = (self._locate_handle)(search_type, protocol, search_key, &mut buf_size, buf);
        if res != Status::BufferTooSmall {
            return Err(res);
        }

        // Allocate an appropriately-sized buffer and make the call again
        let buf = self.allocate_pool(MemoryType::LoaderData, buf_size)? as *mut Handle;
        (self._locate_handle)(search_type, protocol, search_key, &mut buf_size, buf)
            .as_result()?;

        // Return a slice over the contents
        let num_handles = buf_size / mem::size_of::<Handle>();
        unsafe {
            Ok(Pool::new_unchecked(
                slice::from_raw_parts_mut(buf, num_handles),
                self
            ))
        }
    }

    /// Opens the specified protocol on behalf of the calling agent
    pub fn open_protocol<T>(
        &self,
        handle: Handle,
        agent_handle: Handle,
        controller_handle: Handle,
        attributes: OpenProtocolAttributes
    ) -> Result<EfiBs<T>, Status>
    where T: Protocol {

        let mut interface = unsafe { EfiBs::new() };
        (self._open_protocol)(
            handle,
            T::guid(),
            &mut interface,
            agent_handle,
            controller_handle,
            attributes
        )
            .as_result()?;

        if interface.is_null() {
            Err(Status::NotFound)
        } else {
            Ok(unsafe { mem::transmute(interface) })
        }
    }

    /// Closes the specified protocol that was previously opened on the specified `handle`
    pub fn close_protocol<T>(
        &self,
        handle: Handle,
        _interface: EfiBs<T>,
        agent_handle: Handle,
        controller_handle: Handle
    ) -> Result<(), Status>
    where T: Protocol {

        (self._close_protocol)(handle, T::guid(), agent_handle, controller_handle)
            .as_result()
            .map(|_| ())
    }
}
