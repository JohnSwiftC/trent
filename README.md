# trent

A fun P2P file hosting application.

## Current Progress

### File Sharing

File sharing is functionally complete. A server can host multiple files which are then served to client either with or without zstd compression. This is done with a custom binary format which looks something like the following:

> All fields in any of trent's formats are big endian.

**Prelude**

- Version (4 bytes)
- Flags (4 bytes, value of 1 indicates a compressed file, while 0 indicates it is not compressed.)
- Chunks (4 bytes)
- Chunk Size (4 bytes)
- Last Chunk Size (4 bytes)

**Exchange**

- Client sends requested chunk number (0 indexed, 4 bytes)
- Server sends exactly as many bytes are within the requested chunk. The client is responsible for ensuring it allocates a large enough buffer depending on if the chunk is a normal or final chunk.

The client will then decompress the downloaded file if neccessary.

### Routing

while partially unfinished, a client will be able to direct the server action with the following format:

- Version (4 bytes)
- Action (4 bytes, 0 is currently download file, 1 is send available files list)

This will then lead the server immediately into the desired action. I'll almost definitely change this later because it feels really finicky.

## Future Plans / Requirements

- Encryption (Obv)
- Allow servers to send peer addresses to clients.
