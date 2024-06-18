#![allow(non_camel_case_types, clippy::upper_case_acronyms, non_snake_case)]
mod types {
	pub type NTSTATUS = i32;
	pub type ULONG = u32;
	pub type PULONG = *mut ULONG;
	pub type ULONG_PTR = usize;
	pub type LONG = i32;
	pub type WCHAR = u16;
	pub type USHORT = u16;
	pub type UCHAR = u8;
	pub type UINT = u32;
	pub type DWORD = u32;
	pub type HANDLE = *mut std::ffi::c_void;
	pub type HMODULE = isize;
	pub type PVOID = *mut std::ffi::c_void;
	pub type PWSTR = *mut u16;
	pub type BOOL = bool;
	pub type PROCESSINFOCLASS = i32;
}

mod constants {
	use super::types::*;
	pub const STATUS_SUCCESS: NTSTATUS = 0x00000000;
	pub const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
	pub const MAX_PATH: usize = 256;
	pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
	pub const PROCESS_TERMINATE: DWORD = 0x00000001;
	pub const CREATE_NEW_CONSOLE: DWORD = 0x00000010;
	pub const CREATE_NO_WINDOW: DWORD = 0x08000000;
}

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

	#[repr(C)]
	pub struct PROCESS_BASIC_INFORMATION {
		pub exitStatus: NTSTATUS,
		pub pebBaseAddress: PVOID,
		pub affinityMask: usize,
		pub basePriority: i32,
		pub uniqueProcessId: ULONG_PTR,
		pub inheritedFromUniqueProcessId: ULONG_PTR,
	}
}

mod prototypes {
	use super::{structs::*, types::*};
	extern "C" {
		pub fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOEXW) -> NTSTATUS;
		pub fn CreateToolhelp32Snapshot(dwflags: DWORD, th32ProcessID: DWORD) -> HANDLE;
		pub fn Process32First(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32) -> BOOL;
		pub fn Process32Next(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32) -> BOOL;
		pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;
		pub fn ExitProcess(uExitCode: UINT) -> !;
		pub fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
		pub fn GetCurrentProcess() -> HANDLE;
		pub fn GetModuleBaseNameW(hProcess: HANDLE, hModule: HMODULE, lpBaseName: PWSTR, nSize: DWORD) -> DWORD;
		pub fn NtQueryInformationProcess(
			processHandle: HANDLE, processInformationClass: PROCESSINFOCLASS, processInformation: PVOID,
			processInformationLength: ULONG, returnLength: PULONG,
		) -> NTSTATUS;
		pub fn CloseHandle(hObject: HANDLE) -> BOOL;
	}
}

pub use constants::*;
pub use prototypes::*;
pub use structs::*;
pub use types::*;
