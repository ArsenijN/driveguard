# DriveGuard

Automatic USB Drive Backup Tool for Windows

## Version 0.1.0 - Bare Bones

This is the initial "bare bones" version with core functionality.

## Features (v0.1.0)

✅ System tray application
✅ Drive detection (by partition serial number and `.driveGuardID` file)
✅ Basic configuration management (`settings.toml`)
✅ Backup schedule system
✅ File copying with folder structure preservation
✅ i18n support framework (English included)

## Planned Features (Future Versions)

⏳ Countdown window before backup starts
⏳ Old backup cleanup with free space management
⏳ Error handling, retry logic, and detailed log files
⏳ Settings GUI window
⏳ Multiple schedule support (daily + weekly backups)
⏳ Backup verification with checksums
⏳ Interactive setup wizard
⏳ Animated tray icons
⏳ Custom notification settings

## Building

Make sure you have Rust installed, then:

```bash
cargo build --release
```

The executable will be in `target/release/driveguard.exe`

## Configuration Files

- `settings.toml` - Main configuration file
- `schedules/` - Directory containing backup schedules and backup lists
- `schedules/schedule_XXXXX_backup_list.txt` - List of folders to backup for each schedule

## Usage

1. Run `driveguard.exe` - it will appear in the system tray
2. Right-click the tray icon to access settings
3. Configure your backup schedules in `settings.toml`
4. Add folders to backup in the corresponding `backup_list.txt` files

## Drive Identification

DriveGuard can identify drives in two ways:

1. **Partition Serial Number** - More reliable, automatically detected
2. **`.driveGuardID` file** - Place this file at the root of your drive with a unique ID

## Configuration Example

```toml
[general]
language = "en"
min_free_space_gb = 10
warn_before_delete = true

[[schedules]]
id = "schedule_1700000000"
name = "Weekly USB Backup"
enabled = true
drive_serial = "1234567890"
drive_id_file = true
source_paths = []
destination_path = "E:\\Backups"
interval_days = 7
last_backup = "2025-11-19T12:00:00Z"
trigger_on_connect = true
trigger_on_schedule = false
countdown_minutes = 5
```

## Backup List Format

Edit `schedules/schedule_XXXXX_backup_list.txt`:

```
# DriveGuard Backup List
# Add one path per line

C:\Users\YourName\Documents
C:\Users\YourName\Pictures
D:\ImportantData
```

## License

MIT License - Feel free to use and modify

## Contributing

This is the bare bones version. More features coming soon!
Contributions welcome on GitHub.

## Requirements

- Windows 10 or later
- Rust 1.70+ (for building from source)

## Known Limitations (v0.1.0)

- No GUI for schedule management (edit TOML files manually)
- No countdown window (backups start immediately when due)
- No backup verification
- Basic error handling
- No cleanup of old backups yet

These will be addressed in future versions!
