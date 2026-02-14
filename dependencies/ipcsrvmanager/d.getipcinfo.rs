/* This is the main dependency for GetIPCTargetingInfo & GetIPCPagingInfo.
It Includes the Main GetIPCTargetingInfo & GetIPCPagingInfo inside this file.
*/

// d.getipcinfo.rs - IPC Info Query Interface for Lumen OS (Nexus 6 ARMv7a)
// Main dependency providing GetIPCTargetingInfo & GetIPCPagingInfo
// Direct access to IpcPagingHandler + IpcTargetingControl + OSServer state

use core::slice;
use core::ptr;
use alloc::string::String;
use alloc::format;

use crate::handler::{IpcPagingHandler, IpcTargetingControl};
use crate::osserver::{OSServer, OSServerState, OSServerConfig};
use crate::main::{MEM_PAGE_SIZE, SYSTEM_UID, LUMEN_MAGIC, NEXUS6_CODENAME, KERNEL_BASE};

#[repr(C)]
pub struct IPCInfo {
    pub magic: u32,
    pub version: u32,
    pub mem_page_size: usize,
    pub uid_enforced: u32,
    pub target_device: [u8; 64],
    pub pages_mapped: u32,
    pub active_transactions: u32,
    pub lock_type: [u8; 16],
    pub binder_handle: u32,
    pub server_port: u32,
    pub valid: bool,
}

#[repr(C)]
pub struct IPCTargetingInfo {
    pub target_device: [u8; 64],
    pub allowed_uids: [u32; 8],
    pub uid_count: u32,
    pub target_valid: bool,
    pub enforced: bool,
    pub lock_active: bool,
}

#[repr(C)]
pub struct IPCPagingInfo {
    pub page_size: usize,
    pub pages_mapped: u32,
    pub l1_table_base: usize,
    pub total_ipc_region: usize,
    pub cache_flags: u32,
    pub tlb_flushes: u32,
    pub valid: bool,
}

pub struct GetIPCInfo;

static mut GLOBAL_IPC_INFO: IPCInfo = IPCInfo {
    magic: LUMEN_MAGIC,
    version: 0x10001,
    mem_page_size: MEM_PAGE_SIZE,
    uid_enforced: SYSTEM_UID,
    target_device: *const u8 as [u8; 64],  // Filled at init
    pages_mapped: 0,
    active_transactions: 0,
    lock_type: *const u8 as [u8; 16],     // "Global"
    binder_handle: 0xDEADBEEF,
    server_port: 0x4C495043,
    valid: false,
};

impl GetIPCInfo {
    pub const fn new() -> Self {
        Self
    }

    /// Main entry point: Get complete IPC info (combines targeting + paging)
    pub fn get_ipc_info(&self, uid: u32) -> IPCInfo {
        if !self.validate_uid(uid) {
            return self.invalid_ipc_info();
        }
        
        unsafe {
            let paging_info = self.get_ipc_paging_info();
            let targeting_info = self.get_ipc_targeting_info();
            
            GLOBAL_IPC_INFO.pages_mapped = paging_info.pages_mapped;
            GLOBAL_IPC_INFO.valid = true;
            
            // Copy targeting data
            let target_slice = NEXUS6_CODENAME.as_bytes();
            GLOBAL_IPC_INFO.target_device[..target_slice.len()].copy_from_slice(target_slice);
            
            let lock_slice = b"Global";
            GLOBAL_IPC_INFO.lock_type[..lock_slice.len()].copy_from_slice(lock_slice);
            
            GLOBAL_IPC_INFO
        }
    }
    
    /// GetIPCTargetingInfo - Complete targeting subsystem info
    pub fn get_ipc_targeting_info(&self) -> IPCTargetingInfo {
        let mut info = IPCTargetingInfo {
            target_device: *b"Motorola Nexus 6 shamu                                                                                                    ",
            allowed_uids: [SYSTEM_UID, 1001, 0, 0, 0, 0, 0, 0],
            uid_count: 2,
            target_valid: true,
            enforced: true,
            lock_active: false,
        };
        
        // OSServer targeting validation
        unsafe {
            if let Some(osserver) = crate::osserver::OS_SERVER {
                info.target_valid = (*osserver).validate_target(NEXUS6_CODENAME);
            }
        }
        
        info
    }
    
    /// GetIPCPagingInfo - Complete paging subsystem info  
    pub fn get_ipc_paging_info(&self) -> IPCPagingInfo {
        IPCPagingInfo {
            page_size: MEM_PAGE_SIZE,
            pages_mapped: 64,  // 1MB IPC region / 16kb
            l1_table_base: KERNEL_BASE + 0x4000,
            total_ipc_region: MEM_PAGE_SIZE * 64,
            cache_flags: 0x5,  // WBWA
            tlb_flushes: 0,
            valid: true,
        }
    }
    
    /// Direct OSServer state access
    pub fn get_osserver_state(&self) -> Option<&'static OSServerState> {
        unsafe {
            crate::osserver::OS_SERVER
                .as_ref()
                .map(|server| (*server).state)
        }
    }
    
    fn validate_uid(&self, uid: u32) -> bool {
        uid == SYSTEM_UID || uid == 1001  // system + radio
    }
    
    fn invalid_ipc_info(&self) -> IPCInfo {
        IPCInfo {
            magic: 0xDEADBEEF,
            version: 0,
            mem_page_size: 0,
            uid_enforced: 0,
            target_device: [0u8; 64],
            pages_mapped: 0,
            active_transactions: 0,
            lock_type: [0u8; 16],
            binder_handle: 0,
            server_port: 0,
            valid: false,
        }
    }
}

// ======================== C EXPORT INTERFACE ========================

#[no_mangle]
pub extern "C" fn get_ipc_info(uid: u32) -> *mut IPCInfo {
    static mut INFO_CACHE: IPCInfo = IPCInfo {
        magic: LUMEN_MAGIC,
        version: 0x10001,
        mem_page_size: MEM_PAGE_SIZE,
        uid_enforced: SYSTEM_UID,
        target_device: [0u8; 64],
        pages_mapped: 0,
        active_transactions: 0,
        lock_type: [0u8; 16],
        binder_handle: 0xDEADBEEF,
        server_port: 0x4C495043,
        valid: false,
    };
    
    let getter = GetIPCInfo::new();
    let info = getter.get_ipc_info(uid);
    
    unsafe {
        core::ptr::copy_nonoverlapping(
            &info as *const IPCInfo as *const u8,
            &mut INFO_CACHE as *mut IPCInfo as *mut u8,
            core::mem::size_of::<IPCInfo>()
        );
        &mut INFO_CACHE
    }
}

#[no_mangle]
pub extern "C" fn get_ipc_targeting_info() -> IPCTargetingInfo {
    GetIPCInfo::new().get_ipc_targeting_info()
}

#[no_mangle]
pub extern "C" fn get_ipc_paging_info() -> IPCPagingInfo {
    GetIPCInfo::new().get_ipc_paging_info()
}

#[no_mangle]
pub extern "C" fn dump_ipc_info() {
    let info = GetIPCInfo::new().get_ipc_info(SYSTEM_UID);
    
    unsafe {
        crate::main::lumen_os_println("=== Lumen IPC Info (Nexus 6 shamu) ===");
        crate::main::lumen_os_println(&alloc::format!(
            "Magic: 0x{:X} | Page Size: {} bytes", 
            info.magic, info.mem_page_size
        ));
        crate::main::lumen_os_println(&alloc::format!(
            "UID: {} | Target: {}", 
            info.uid_enforced,
            core::str::from_utf8(&info.target_device).unwrap_or("unknown")
        ));
        crate::main::lumen_os_println(&alloc::format!(
            "Pages: {} | Lock: {}", 
            info.pages_mapped,
            core::str::from_utf8(&info.lock_type).unwrap_or("unknown")
        ));
    }
}

// ======================== BINDER SERVICE INTEGRATION ========================

#[no_mangle]
pub extern "C" fn binder_get_ipc_info_handler(txn: *mut crate::main::BinderTransaction) -> i32 {
    unsafe {
        let txn = &mut *txn;
        if txn.sender_uid != SYSTEM_UID && txn.sender_uid != 1001 {
            return -13;  // EACCES
        }
        
        let info_ptr = get_ipc_info(txn.sender_uid);
        txn.data_ptr = info_ptr as *mut u8;
        txn.data_size = core::mem::size_of::<IPCInfo>();
        
        0  // Success
    }
}

/// Initialize IPC info cache at boot
#[no_mangle]
pub extern "C" fn init_ipc_info_cache() {
    let info = GetIPCInfo::new().get_ipc_info(SYSTEM_UID);
    unsafe {
        crate::main::lumen_os_println("IPC Info cache initialized");
    }
}

// ======================== IPC MONITORING SYSTEM ========================

/// Real-time IPC monitoring + statistics collection
#[repr(C)]
pub struct IPCStats {
    pub transactions_total: AtomicU32,
    pub pages_mapped_total: AtomicU32,
    pub lock_contention: AtomicU32,
    pub uid_violations: AtomicU32,
    pub target_mismatches: AtomicU32,
    pub tlb_flushes_total: AtomicU32,
    pub max_transaction_size: AtomicU32,
    pub avg_latency_ns: AtomicU32,
    pub uptime_seconds: AtomicU32,
    pub errors_total: AtomicU32,
}

pub static mut IPC_MONITOR: IPCStats = IPCStats {
    transactions_total: AtomicU32::new(0),
    pages_mapped_total: AtomicU32::new(0),
    lock_contention: AtomicU32::new(0),
    uid_violations: AtomicU32::new(0),
    target_mismatches: AtomicU32::new(0),
    tlb_flushes_total: AtomicU32::new(0),
    max_transaction_size: AtomicU32::new(0),
    avg_latency_ns: AtomicU32::new(0),
    uptime_seconds: AtomicU32::new(0),
    errors_total: AtomicU32::new(0),
};

static mut BOOT_TIMESTAMP: u64 = 0;

impl GetIPCInfo {
    /// Initialize IPC monitoring system at boot (runs after cache init)
    pub fn init_ipc_monitoring(&self) {
        unsafe {
            BOOT_TIMESTAMP = self.rdtsc();  // ARMv7 timestamp counter
            
            // Zero stats
            IPC_MONITOR.transactions_total.store(0, Ordering::Relaxed);
            IPC_MONITOR.pages_mapped_total.store(0, Ordering::Relaxed);
            IPC_MONITOR.lock_contention.store(0, Ordering::Relaxed);
            IPC_MONITOR.errors_total.store(0, Ordering::Relaxed);
            
            // Register monitoring callback with binder driver
            self.register_monitor_callback();
            
            crate::main::lumen_os_println("IPC Monitoring initialized - real-time stats enabled");
        }
    }
    
    #[inline(always)]
    fn rdtsc(&self) -> u64 {
        let mut lo: u32; let mut hi: u32;
        unsafe {
            core::arch::asm!(
                "mrrc p15, 0, {0}, {1}, c15",  // CNTVCT_EL0 (Nexus 6 ARMv7)
                out(reg) lo,
                out(reg) hi,
                options(nostack)
            );
        }
        ((hi as u64) << 32) | (lo as u64)
    }
    
    fn register_monitor_callback(&self) {
        unsafe {
            // Hook into binder transaction completion
            core::ptr::write_volatile(
                0x8000_4000 as *mut unsafe extern "C" fn(*mut crate::main::BinderTransaction),
                Some(monitor_transaction_hook)
            );
        }
    }
    
    /// Get comprehensive real-time IPC statistics
    pub fn get_ipc_stats(&self) -> IPCStats {
        unsafe { IPC_MONITOR }
    }
    
    /// Dump full monitoring report
    pub fn dump_monitoring_report(&self) {
        unsafe {
            let stats = &IPC_MONITOR;
            let uptime = (self.rdtsc() - BOOT_TIMESTAMP) / 1_000_000;  // ~ns to seconds
            
            crate::main::lumen_os_println("=== Lumen IPC Monitoring Report ===");
            crate::main::lumen_os_println(&alloc::format!(
                "Uptime: {}s | Transactions: {}", 
                uptime, stats.transactions_total.load(Ordering::Relaxed)
            ));
            crate::main::lumen_os_println(&alloc::format!(
                "Pages mapped: {} | TLB flushes: {}", 
                stats.pages_mapped_total.load(Ordering::Relaxed),
                stats.tlb_flushes_total.load(Ordering::Relaxed)
            ));
            crate::main::lumen_os_println(&alloc::format!(
                "Errors: {} (UID: {}, Target: {})", 
                stats.errors_total.load(Ordering::Relaxed),
                stats.uid_violations.load(Ordering::Relaxed),
                stats.target_mismatches.load(Ordering::Relaxed)
            ));
            crate::main::lumen_os_println(&alloc::format!(
                "Lock contention: {}", 
                stats.lock_contention.load(Ordering::Relaxed)
            ));
        }
    }
}

/// Monitoring hook for every binder transaction
#[no_mangle]
pub unsafe extern "C" fn monitor_transaction_hook(txn: *mut crate::main::BinderTransaction) {
    let txn = &*txn;
    
    // Update transaction count
    IPC_MONITOR.transactions_total.fetch_add(1, Ordering::Relaxed);
    
    if txn.data_size > IPC_MONITOR.max_transaction_size.load(Ordering::Relaxed) as usize {
        IPC_MONITOR.max_transaction_size.store(txn.data_size as u32, Ordering::Relaxed);
    }
    
    // Error counters
    if txn.return_error != 0 {
        IPC_MONITOR.errors_total.fetch_add(1, Ordering::Relaxed);
        match txn.return_error {
            -13 => IPC_MONITOR.uid_violations.fetch_add(1, Ordering::Relaxed),  // EACCES
            -99 => IPC_MONITOR.target_mismatches.fetch_add(1, Ordering::Relaxed), // LIPC_TARGET_MISMATCH
            _ => {}
        }
    }
}

/// Periodic stats sampler (called from main loop)
#[no_mangle]
pub extern "C" fn sample_ipc_stats() {
    unsafe {
        IPC_MONITOR.uptime_seconds.fetch_add(1, Ordering::Relaxed);
    }
}

/// C-callable stats dump
#[no_mangle]
pub extern "C" fn dump_ipc_monitoring() {
    GetIPCInfo::new().dump_monitoring_report();
              }
          
