b rust_main
# b src/syscall/mod.rs:70 if syscall_id == 64
b src/syscall/mod.rs:75 if syscall_id == 169
b src/syscall/process.rs:73
define dss
  dashboard source -output /dev/pts/$arg0
  dashboard source -style height 0
end

define dsa
  dashboard assembly -output /dev/pts/$arg0
  dashboard assembly -style height 0
end