#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod types {
	pub type NTSTATUS = i32;
	pub type ULONG = u32;
	pub type ULONG_PTR = usize;
	pub type LONG = i32;
	pub type WCHAR = u16;
	pub type USHORT = u16;
	pub type UCHAR = u8;
	pub type DWORD = u32;
	pub type HANDLE = *mut std::ffi::c_void;
	pub type BOOL = bool;
}

#[allow(non_snake_case, unused, clippy::upper_case_acronyms)]
mod constants {
	use super::types::*;
	pub const STATUS_SUCCESS: NTSTATUS = 0x00000000;
	pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
	pub const MAX_PATH: usize = 256;
	pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
}

#[allow(non_snake_case, clippy::upper_case_acronyms)]
mod structs {
	use super::{constants::*, types::*};

	#[repr(C)]
	pub struct OSVERSIONINFOEXW {
		pub dwOSVersionInfoSize: ULONG,
		pub dwMajorVersion: ULONG,
		pub dwMinorVersion: ULONG,
		pub dwBuildNumber: ULONG,
		pub dwPlatformId: ULONG,
		pub szCSDVersion: [WCHAR; 128],
		pub wServicePackMajor: USHORT,
		pub wServicePackMinor: USHORT,
		pub wSuiteMask: USHORT,
		pub wProductType: UCHAR,
		pub wReserved: UCHAR,
	}

	#[repr(C)]
	pub struct PROCESSENTRY32 {
		pub dwSize: DWORD,
		pub cntUsage: DWORD,
		pub th32ProcessID: DWORD,
		pub th32DefaultHeapID: ULONG_PTR,
		pub th32ModuleID: DWORD,
		pub cntThreads: DWORD,
		pub th32ParentProcessID: DWORD,
		pub pcPriClassBase: LONG,
		pub dwFlags: DWORD,
		pub szExeFile: [u8; MAX_PATH],
	}
}

mod prototypes {
	use super::{structs::*, types::*};
	extern "C" {
		pub fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOEXW) -> NTSTATUS;
		pub fn CreateToolhelp32Snapshot(dwflags: DWORD, th32ProcessID: DWORD) -> HANDLE;
		pub fn Process32First(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32) -> BOOL;
		pub fn Process32Next(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32) -> BOOL;
	}
}

pub use constants::*;
pub use prototypes::*;
pub use structs::*;
