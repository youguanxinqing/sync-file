
# Usage

## Start A Server

Firstly, start `sync-server` process:
```bash
./sync-server
```

## Ping

```bash
$ ./target/release/sync-client --addr 127.0.0.1:9091 test --ping
# output
[2025-05-02 17:35:34 INFO sync_client] code: 200 OK, Ping OK!!!
```

## Upload

Secondly, send a request by `sync-client`:
```bash
./sync-client --addr [remote_host]:[remote_port] --local-file-path [local_file_path] --remote-file-path [remote_file_path]
```

```bash
./sync-client --addr [remote_host]:[remote_port] --file-mappings [local_file1]:[remote_file1],[local_file2]:[remote_file2],...
```

Note that `sync-server` backups a file to a hiden directory named `.[nearest_parent_directory]` when your request mode is `safe` every time.

## Download

You can also donwload a file from remote devices.
```bash
./sync-client --addr [remote_host]:[remote_port] pull --file-mappings [local_file1]:[remote_file1],[local_file2]:[remote_file2],...
```
