# trent

A robust and fast P2P file hosting application.

## Premise

The application has two main features: fast file uploads/downloads, and the optional sharing of peer lists to check for other downloadable files. File sharing is the current development focus. Once this system is in place, I plan on exploring the P2P aspects of the project.

## File Sharing

Trent optimizes for the time spent moving a file across a network. This is mainly accomplished with the use of zstd for compression between peers and the QUIC protocol for networking.

Files being hosted by a peer are loaded and compressed at start-up, removing any additional work when transmitting files. Files are also chunked when loaded, allowing for partial downloads.
