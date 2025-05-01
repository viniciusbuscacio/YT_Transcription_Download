# YouTube Transcription Downloader

YouTube Transcription Downloader is a command-line application written in Rust to download transcriptions from YouTube videos.

## Features

- Download transcriptions from individual YouTube videos.
- Support for batch downloading from multiple URLs.
- Download transcriptions from all videos in a YouTube channel.
- Choose the transcription language or use automatic detection.
- Formatted transcriptions with timestamps.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo installed (for developers).

## Installation for Non-Developers

For non-developers, you can download the pre-built executable from the `installer` folder and run it directly:

[Download YT_Transcription_Download.exe](installer/YT_Transcription_Download.exe)

## Installation for Developers

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/yt_transcription_downloader.git
   ```
2. Navigate to the project directory:
   ```bash
   cd yt_transcription_downloader
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```

The executable will be available in the `target/release` directory with the name `YT_Transcription_Download.exe` (Windows).

## Usage

Run the program in the terminal:

```bash
./target/release/YT_Transcription_Download.exe
```

### Examples of Usage

1. Download the transcription of a single video:
   ```
   https://www.youtube.com/watch?v=VIDEO_ID
   ```

2. Download transcriptions of multiple videos:
   ```
   https://www.youtube.com/watch?v=ID1 https://www.youtube.com/watch?v=ID2
   ```

3. Download transcriptions of all videos from a channel:
   ```
   https://www.youtube.com/@channel_name
   ```

### Output

Transcriptions will be saved in the `Downloads` directory with the following format:
- For individual videos: `Downloads/[video_id]_[language]_transcript.txt`
- For channel videos: `Downloads/[channel_id]/[video_id]_[language]_transcript.txt`

Each transcription includes timestamps in the `[MM:SS]` format.

## Contribution

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License. See the LICENSE file for details.