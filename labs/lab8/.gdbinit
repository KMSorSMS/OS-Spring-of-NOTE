b rust_main
# b src/syscall/mod.rs:70 if syscall_id == 64
# b src/syscall/mod.rs:72 if syscall_id == 169
# ignore $bpnum 23
# b src/syscall/process.rs:73
# b src/syscall/process.rs:131
# b src/syscall/mod.rs:82 if syscall_id == 222 || syscall_id == 215
# ignore $bpnum 8
# b src/syscall/process.rs:233
# b src/task/processor.rs:67
# b sys_spawn
# b sys_waitpid
# b sys_fstat
# b sys_unlinkat
# b sys_semaphore_create
# b sys_semaphore_up
# b sys_semaphore_down
# b src/syscall/sync.rs:255
# ignore $bpnum 10
# b sys_semaphore_create
b update_rsc_down
ignore $bpnum 12
# b src/syscall/deadlock.rs:81
define dss
  dashboard source -output /dev/pts/$arg0
  dashboard source -style height 0
end

define dsa
  dashboard assembly -output /dev/pts/$arg0
  dashboard assembly -style height 0
end