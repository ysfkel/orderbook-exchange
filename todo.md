For a low-latency exchange system, eventually you will likely want:

busy-spin instead of sleeping
SPSC ring buffer between Aeron and matching engine
pinned threads
batching fragments
zero-copy parsing
fixed-size binary messages instead of UTF-8 strings