# Base-UTF8
Base-UTF8 is a specially designed encoding method used to store arbitrary binary data in environments that only support UTF-8 encoding.

1. Add a reserved byte at the beginning of the data to store the length of padding to be added later.
2. Pad the end of the data with zeros until its length is a multiple of 7. This is done to ensure that the encoding can be divided into blocks of 7 bytes each.
3. Store the length of the padding in the reserved byte added in step 1.
4. Divide the padded data into blocks of 7 bytes each.
5. Encode each block by storing the first bit of each byte in the original block in the last 7 bits of the first byte of the encoded block. The remaining 7 bytes of the encoded block store each byte in the original block where the first bit has been set to 0.
6. Output the encoded data.
