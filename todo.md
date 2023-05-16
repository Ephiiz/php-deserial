# Todo

This project was originally an attempt to just learn how to use nom by writing a parser for the PHP serialization format, however
the next steps is going to be writing a transformer for the parser result to fully deserialize into a struct. The goal is to do so without
using serde at all, mainly just because its for learning, so the following is the remaining steps to finish the deserialization procces. After that
is writing a serializer to output the PHP serialization format, might use serde for this process, not sure yet.


## Segments

### Transformer
[ ] - Write scaffold
