# trent

A fun P2P file hosting application.

## Usage

### Serving Files

> Everything here is also available via the clap parser, see trent --help.

trent is built to act as both the client and server, to start a server, first write a config.yaml to configure files to host.

```yaml
files:
  - dogdog:
      path: dogdog.jpg
      compressed: false

  - video:
      path: "newvid.mp4"
      compressed: true
```

Here, dogdog and video are the names your server will advertise when requested to share files, and the name used when a client requests a download.

> Note, the file path is relative to where trent is being ran. Absolute paths can be used in their place.

> compressed enables built in zstd compression when sending a file.

To start serving files,

`trent -p peers.db -c config.yaml --bind 0.0.0.0:5000`

> Currently, specifying a peers.db is required, however the server usage of the database is not finished, so this is just visual.

### Client Peerlists

The client _is_ currently built around a peer database. The peer db stores known peers and can be modified in the following ways,

`trent -p peers.db add-peer --name <NAME> --host <ADDR:PORT> --ty <lan|public|vpn>`

Here, we specify the peer db to operate on. We then add a peer with a name, an address and port, and a type. The type is used internally, so a known peer can be accessed

with a LAN address, public address, or VPN address.

`trent -p peers.db remove-peer --name <NAME>`

Here you can remove a peer from a peer db with the peer's name.

`trent -p peers.db view-peers`

Finally, you can see the peers stored in a database.

### Downloading Files

trent allows you to request the names and information of files being hosted of peers in your peerlist,

`trent -p peers.db files`

Then, you can request a file to be downloaded by name,

`trent -p peers.db download --peer <PEER NAME> --file <FILE TO DOWNLOAD> --output <PATH TO DOWNLOADED FILE>`

## Development Notes

### Peers Module

The peers module exposes PeerStore and PeerExport structs for managing peers. The peer type currently is used to filter the order that peers are returned, for example, requesting a peer list from PeerStore with RequestView::LAN will still return the public and VPN addresses of that peer, they will just appear after the LAN entry.

PeerExport handles the server up check, it will attempt to connect and perform the is_alive handshake with a server before returning Result<TcpStream\> to the caller with a specified timeout duration. Sometime in the future I will allow this time to be configured, but it's currently 2 seconds for a valid socket and 2 seconds for a successful handshake.

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

The client will then decompress the downloaded file if necessary.

> There are other protocols currently in trent, such as the required version exchange before _any_ client server interaction, but I am not writing them here because they will almost definitely change in the future. Downloading may be the only stablish one.

### Routing

A client is able to direct the server action with the following format:

- Version (4 bytes)

- Action (4 bytes, 0 is currently download file, 1 is send available files list, 2 is a simple handshake)

This will then lead the server immediately into the desired action. I'll almost definitely change this later because it feels really finicky.

## Future Plans / Requirements

- Encrypted Streams
- Requesting a server's peer list and merging it into your own. This is hopefully the coolest part of the project and was the main idea I had in mind when starting this one.
- Refactoring
- QOL things
