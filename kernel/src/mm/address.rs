use crate::config::PAGE_SIZE;

use super::page_table::PageTableEntry;

// 对于 SV39 这几个值是固定的
const VA_WIDTH_SV39: usize = 39;
const PA_WIDTH_SV39: usize = 56;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        // 只取低 56 位
        Self(v & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl Into<usize> for PhysAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        // 只取低 (56 - 12) 位
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl Into<usize> for PhysPageNum {
    fn into(self) -> usize {
        self.0
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        // 只取低 39 位
        Self(v & ((1 << VA_WIDTH_SV39) - 1))
    }
}

impl Into<usize> for VirtAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        // 只取低 (56 - 12) 位
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl Into<usize> for VirtPageNum {
    fn into(self) -> usize {
        self.0
    }
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        // 取低 12 位
        self.0 & ((1 << PAGE_SIZE) - 1)
    }
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / (1 << PAGE_SIZE))
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + (1 << (PAGE_SIZE - 1))) / (1 << PAGE_SIZE))
    }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & ((1 << PAGE_SIZE) - 1)
    }

    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / (1 << PAGE_SIZE))
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + (1 << (PAGE_SIZE - 1))) / (1 << PAGE_SIZE))
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        // 判断一下偏移量是否为 0
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        // 判断一下偏移量是否为 0
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE)
    }
}

impl PhysPageNum {
    pub fn get_pte_entry(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512) }
    }

    pub fn get_page_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4 * 1024) }
        //4KiB 一个页的大小
    }

    pub fn get_immut<T>(&self) -> &T {
        let pa: PhysAddr = self.clone().into();
        unsafe { &*(pa.0 as *const T) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        // 解析 三级 ppn
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 0x1FF;
            vpn >>= 9;
        }
        idx
    }
}

// 一个简单连续迭代器的实现

pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
/// a simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(start: T, end: T) -> Self {
        // assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
