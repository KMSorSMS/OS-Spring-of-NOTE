use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    /// Create a vfs inode
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }
    /// 访问dirent中的inode_id by name，返回inode_id
    pub fn get_inode_id_by_name(&self, root_inode: &Inode, name: &str) -> Option<u32> {
        // assert it is a directory
        assert!(root_inode.is_dir());
        root_inode.read_disk_inode(|disk_inode| root_inode.find_inode_id(name, disk_inode))
    }

    ///单纯根据inode来计算出inode_id
    pub fn get_inode_id_by_inode(&self) -> u32 {
        let fs = self.fs.lock();
        let inode_id = fs.get_inode_id(self.block_id, self.block_offset);
        inode_id
    }

    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_id() as u32);
            }
        }
        None
    }
    ///判定当前inode是一个文件还是目录，true为dir
    pub fn is_dir(&self) -> bool {
        self.read_disk_inode(|disk_inode| disk_inode.is_dir())
    }
    ///遍历一个为dir的inode，查找它的文件中硬链接的个数
    pub fn find_hard_link(&self, root_inode: &Inode) -> usize {
        //思路是遍历这个inode的所有dirent，查找inode_id 和当前的inode_id相同的inode_id的个数
        //先取得当前inode的inode_id
        let fs = self.fs.lock();
        let inode_id = fs.get_inode_id(self.block_id, self.block_offset);
        //这里可以释放fs的锁，因为我们不会再调用fs的方法
        drop(fs);
        let op = |disk_root_inode: &DiskInode| {
            // assert it is a directory
            assert!(disk_root_inode.is_dir());
            // has the file been created?
            let file_count = (disk_root_inode.size as usize) / DIRENT_SZ;
            let mut count: usize = 0;
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_root_inode.read_at(
                        i * DIRENT_SZ,
                        dirent.as_bytes_mut(),
                        &self.block_device,
                    ),
                    DIRENT_SZ,
                );
                if dirent.inode_id() == inode_id {
                    count += 1;
                }
            }
            //返回计数值
            count
        };
        root_inode.read_disk_inode(op)
    }
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// Create inode under current inode by name
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }
    ///根据传入的inode编号创建一个file，而不是如create那样alloc_inode从文件系统里面分得一个inode
    /// 除此以外和create一样,返回值反映创建成功与否
    pub fn create_file_inode(&self, name: &str, inode_id: u32) -> isize {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return -1;
        }
        // create a new file
        // initialize inode
        // let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(inode_id);
        // get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
        //     .lock()
        //     .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
        //         new_inode.initialize(DiskInodeType::File);
        //     });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        block_cache_sync_all();
        // return success
        0
        // release efs lock automatically by compiler
    }
    /// unlink实现（参考clear），逻辑是先检查是否是最后一个文件
    /// 返回值和syscall的一样，成功0，失败-1
    pub fn unlink(&self, root_inode: &Inode, name: &str) -> isize {
        //检查是否是最后一个文件
        log::info!("\nnow the link is {}\n", self.find_hard_link(root_inode));
        if self.find_hard_link(root_inode) == 1 {
            self.clear();
        }
        // //清空inode,这步操作一直失败，我后面检查实际上这里只是改变了inode没有动对应的内容，但是
        // 我最后推测是在cache sync的时候会由于检测一个inode之前更新了，刷新为0，导致将数据清空，所以这里就不
        // 把这部分inode清除，这是安全的，因为后面重新分配这个inode的时候也会重新填入initialize的值
        // get_block_cache(self.block_id as usize, Arc::clone(&self.block_device))
        //     .lock()
        //     .modify(self.block_offset, |disk_inode: &mut DiskInode| {
        //         disk_inode.initialize(DiskInodeType::File);
        //     });
        // disk_inode.clear_size(&self.block_device);
        //同时对于dirent的处理,找到name符合的dirent，将他除去
        root_inode.modify_disk_inode(|root_inode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let mut dirent = DirEntry::empty();
            for i in 0..file_count {
                assert_eq!(
                    root_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                if dirent.name() == name {
                    //找到了，将其删除
                    let new_dirent = DirEntry::empty();
                    root_inode.write_at(DIRENT_SZ * i, new_dirent.as_bytes(), &self.block_device);
                    break;
                }
            }
        });
        block_cache_sync_all();
        log::info!("\nnow the link is {}\n", self.find_hard_link(root_inode));
        0
    }

    ///返回block_id
    pub fn get_block_id(&self) -> usize {
        self.block_id
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
