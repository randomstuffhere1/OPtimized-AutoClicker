# (OP)timized Auto Clicker v2.0.0
*Like the [original](https://opautoclicker.dev/), but written in Rust!*

A fast, ultra-precise auto clicker of Rust for Windows. It features advanced timing algorithms, customizable click patterns, and a native Windows GUI interface.

## Features

* **Ultra-Precise Timing**: Employs CPU-specific optimizations and high-resolution timers for sub-millisecond accuracy
* **Multiple Click Types**: Single click, double click support
* **Mouse Button Support**: Left, right, and middle mouse buttons
* **Flexible Intervals**: Define time in hours, minutes, seconds, and milliseconds
* **Random Intervals**: Random intervals can be introduced into click intervals for natural behavior
* **Position Control**: Click at the current cursor position or fixed coordinates
* **Repeat Options**: Indefinitely, a specific number of times, for a specified duration or while the hotkey is held down clicking is possible
* **Global Hotkeys**: Clicking can be started/stopped by customizable hotkeys
* **Persistent Settings**: Your configuration is automatically saved and restored
* **Administrator Detection**: Gives a warning when admin rights may be needed for certain applications
* **Safety Disable**: If the mouse appears stuck in a corner, the clicking is automatically stopped
* **Performance Monitoring**: On-board performance counters for optimization

## System Requirements

* **Operating System**: Windows 10 or later
* **Architecture**: x86_64 (64-bit)
* **Permissions**: Administrator rights are recommended for compatibility with games and protected applications

## Building

### Prerequisites

* Rustup (https://rustup.rs/)

### Build Commands
```bash
# Build release version
cargo build --release

# Build debug version
cargo build

# Run tests
cargo test

# Check code without building
cargo check
```
### Build Scripts

The project has build scripts for easy building:

* **Windows**: `build.bat` - Builds both debug and release versions

Check scripts: `check.bat` for quick validation

These scripts automatically handle both debug and release builds and report success/failure status.

## Downloading Releases

### Github Actions Builds

You can download pre-compiled binaries directly from the Github Actions workflow:

1. Navigate to the "Actions" tab
2. Select the latest successful workflow run
3. Scroll down to the "Artifacts" section
4. Download the Windows release binary

**Note**: Github Actions builds are manually triggered and only compiled when a stable build is ready. These builds represent carefully vetted releases with full optimizations.

## Usage

1. **Launch the Application**: Execute 'rust_autoclicker.exe' (usually found in `target/release` if you built it yourself)
2. **Configure Settings**:
	* Define the click interval by filling in the hours, minutes, seconds, and milliseconds fields
	* Choose the mouse button (Left, Right, or Middle)
	* Select the click type (Single, Double)
	* Configure the repeating options (Finite count, time duration, repeat while held, or indefinite)
	* Set the click position (Current cursor or fixed coordinates)
3. **Start Clicking**: Click on the "Start" button or press the global hotkey (default: F2)
4. **Stop Clicking**: Click on the "Stop" button or press the same hotkey

## Default Mode

For testing or clean configuration, you can launch the program in default mode:
```bash
# Launch with default settings (no config loaded/saved)
rust_autoclicker.exe --default

# or
rust_autoclicker.exe -d
```
If you want to run the program directly, just compile and run:
```bash
# Build and run release version
cargo run --release ---d
```
In default mode:

* Settings are NOT loaded from `autoclicker_settings.dat`
* Settings are NOT saved when the program exits
* Window title shows "(Default Mode)"
* All settings start with hardcoded defaults (100ms interval, Left button, Single click, etc.)

## Repeat While Held Mode

The "Repeat while held" option lets you click continuously as long as you hold the hotkey:

1. In the Click repeat section, select the "Repeat while held" radio button
2. Set your desired click interval and other settings
3. To start clicking, press and hold your hotkey
4. Make the clicking stop immediately by releasing the hotkey

This mode is perfect for:

* Extremely rapid clicking during gaming sessions
* Burst clicking when you need precise control
* Situations where you want to stop clicking instantly without pressing the hotkey again

## Safety Disable Feature

The auto clicker has a safety feature that will stop auto-clicking completely when it finds the mouse pointer is stuck in a corner of the screen:

* **Detection**: When clicking, the cursor position is checked every 100ms
* **Trigger**: After 10 consecutive detections in any corner (within 5 pixels) it activates
* **Action**: The clicking is stopped and a warning window is displayed
* **Configuration**: It can be enabled/disabled from the Settings dialog
* **Purpose**: In case mouse control is lost, it avoids the clicking that is not intended

The feature can be turned off in Settings if it is not needed although it is turned on by default.

## Hotkey Configuration

* Click the "Hotkey" button will bring up the hotkey configuration dialog
* To set a new hotkey, press any key
* The supported keys include F1-F24, alphanumeric keys, and special keys

## Position Picker

1. Choose "Pick Location" radio button
2. Press the "Pick Location" button
3. Now move your mouse to the desired location and click to set it
4. During selection a tooltip will show the current coordinates

## Configuration File

The user's settings get saved automatically to `autoclicker_settings.dat` placed in the application directory. The file consists of:

* Interval settings (hours, minutes, seconds, milliseconds)
* Randomization settings
* Mouse button and click type preferences
* Repeat options (finite count, time duration, repeat while held, or indefinite)
* Time duration settings
* Fixed position coordinates
* Hotkey configuration
* Admin popup suppression setting
* Safety disable setting

## Advanced Features

### Timing Modes

Based on your interval, the application will decide the best timing mode to use:

* **Ultra Precision (< 15ms)**: Pure spin loop is used for maximum accuracy
* **High Precision (< 50ms)**: Hybrid approach with minimal sleep
* **Standard (≥ 50ms)**: Optimized sleep with spin correction

### Zero Millisecond Support

For zero millisecond intervals, the clicker achieves the highest speed through the use of aggressive spin loops with occasional yields so as not to freeze the system.

### Performance Monitoring

The program keeps track of:

* The total number of clicks
* Performance timing checkpoints
* Thread priority status

## Dependencies

* `windows` crate (v0.52) - Windows API bindings
* `rand` crate (v0.8) - Random number generation
* `rand_chacha` crate (v0.3) - ChaCha-based RNG algorithms

## License

This project is offered as-is and intended only for educational and personal use. Please make sure to abide by the terms of service of any apps or games you use this auto clicker with.

## Troubleshooting

### Clicking Doesn't Work in Some Applications

Some applications (especially games) that want to be safe may only accept simulated input if the process sending it is run with administrator privileges. If simulated input is going nowhere, running as administrator might be what you need.

### Timing Inaccuracy

To get the most precise timing possible:

* Close the unnecessary background applications
* Make sure that there are no other processes that consume high CPU and run simultaneously
* Operate at the highest possible thread priority (which is done automatically)

### High CPU Usage

High CPU usage might be the result of the ultra-precise timing modes. This is perfectly normal if the goal is to achieve sub-millisecond accuracy. If CPU utilization is a problem, you may want to use longer intervals instead.

## Technical Details

### Architecture

* **GUI**: Native Windows API (Win32) with custom window procedures
* **Timing**: High resolution timers based on RDTSC with fallback to system timers
* **Threading**: A dedicated click thread runs at a high priority
* **Random Generation**: Custom xoshiro256+ implementation together with Lemire's multiplication method
* **Build System**: Windows icons and version info are compiled from custom resources

### Performance Characteristics

* **Minimum Interval**: 0ms (only limited by CPU speed)
* **Typical Precision**: ±0.1ms for intervals > 1ms
* **Maximum Click Rate**: Up to a few thousand clicks per second (depending on hardware)
* **Memory Usage**: < 10MB typical usage

## Contributing

This is my personal project which primarily deals with performance optimization of the original. Contributors are welcome, especially for:

* Additional platform support
* Implementing future plans
* Any other features that may be useful

## Version History

* **v2.1.1 (Minor)**: Added safety disable feature - automatically stops clicking when mouse appears stuck in corner
* **v2.1.0 (Minor)**: Added repeat while held functionality - click continuously while hotkey is held down
* **v2.0.0 (Major)**: Added time-based repeat functionality - click for a specified duration
* **v1.1.0 (Minor)**: Added application icon and proper version display in window title
* **v1.0.0 (Major)**: Initial release with ultra-precise timing, full feature set, and default mode support

## Future Plans

* Add macro (record/playback) support (Low Priority)