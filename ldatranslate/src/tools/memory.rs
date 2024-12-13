use std::sync::{Arc, Mutex, RwLock, TryLockError};
use std::time::{Duration, Instant};
use byte_unit::UnitType;
use sysinfo::{MemoryRefreshKind, RefreshKind};

#[allow(unused)]
pub struct MemoryReporter {
    sys: Arc<Mutex<sysinfo::System>>,
    interval: Duration,
    last_report: Arc<RwLock<Instant>>,
}


unsafe impl Send for MemoryReporter {}
unsafe impl Sync for MemoryReporter {}

impl MemoryReporter {
    pub fn new(interval: Duration) -> MemoryReporter {
        Self {
            sys: Arc::new(Mutex::new(sysinfo::System::new_with_specifics(
                RefreshKind::nothing().with_memory(MemoryRefreshKind::everything())
            ))),
            last_report: Arc::new(RwLock::new(Instant::now())),
            interval
        }
    }

    fn create_report_string(sys: &mut sysinfo::System) -> String {
        sys.refresh_memory();

        let free_memory = byte_unit::Byte::from_u64(sys.free_memory()).get_appropriate_unit(UnitType::Decimal);
        let available_memory = byte_unit::Byte::from_u64(sys.available_memory()).get_appropriate_unit(UnitType::Decimal);
        let used_memory = byte_unit::Byte::from_u64(sys.used_memory()).get_appropriate_unit(UnitType::Decimal);
        let total_memory = byte_unit::Byte::from_u64(sys.total_memory()).get_appropriate_unit(UnitType::Decimal);
        let free_swap = byte_unit::Byte::from_u64(sys.free_swap()).get_appropriate_unit(UnitType::Decimal);
        let used_swap = byte_unit::Byte::from_u64(sys.used_swap()).get_appropriate_unit(UnitType::Decimal);
        let total_swap = byte_unit::Byte::from_u64(sys.total_swap()).get_appropriate_unit(UnitType::Decimal);

        format!(
            "RAM(available={}, free={}, used={}, total={}), SWAP(free={}, used={}, total={})",
            available_memory,
            free_memory,
            used_memory,
            total_memory,
            free_swap,
            used_swap,
            total_swap,
        )
    }

    pub fn instant_report() -> String {
        Self::create_report_string(&mut sysinfo::System::new())
    }

    pub fn create_report_now(&self) -> String {
        let value = Self::create_report_string(&mut self.sys.lock().unwrap());
        match self.last_report.try_write() {
            Ok(mut value) => {
                *value = Instant::now();
            }
            Err(TryLockError::WouldBlock) => {}
            _ => unreachable!()
        }
        value
    }

    pub fn create_report(&self) -> Option<String> {
        let now = Instant::now();
        {
            let read = self.last_report.read().unwrap();
            if now - *read < self.interval {
                return None;
            }
        }
        let mut sys = match self.last_report.try_write() {
            Ok(mut value) => {
                *value = now;
                self.sys.lock().unwrap()
            }
            Err(TryLockError::WouldBlock) => {
                return None;
            }
            _ => unreachable!()
        };

        sys.refresh_memory();

        let free_memory = byte_unit::Byte::from_u64(sys.free_memory()).get_appropriate_unit(UnitType::Decimal);
        let available_memory = byte_unit::Byte::from_u64(sys.available_memory()).get_appropriate_unit(UnitType::Decimal);
        let used_memory = byte_unit::Byte::from_u64(sys.used_memory()).get_appropriate_unit(UnitType::Decimal);
        let total_memory = byte_unit::Byte::from_u64(sys.total_memory()).get_appropriate_unit(UnitType::Decimal);
        let free_swap = byte_unit::Byte::from_u64(sys.free_swap()).get_appropriate_unit(UnitType::Decimal);
        let used_swap = byte_unit::Byte::from_u64(sys.used_swap()).get_appropriate_unit(UnitType::Decimal);
        let total_swap = byte_unit::Byte::from_u64(sys.total_swap()).get_appropriate_unit(UnitType::Decimal);

        let value = format!(
            "RAM(available={}, free={}, used={}, total={}), SWAP(free={}, used={}, total={})",
            available_memory,
            free_memory,
            used_memory,
            total_memory,
            free_swap,
            used_swap,
            total_swap,
        );

        Some(value)
    }
}
