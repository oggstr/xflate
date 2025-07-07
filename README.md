# XFalte - XML Compression

This is a very rough implementation of [Compression of XML and JSON API Responses](https://ieeexplore.ieee.org/document/9402836).

It's not intended for use. Rather, it's as a means of testing the feasibility of the algorithm for real data.

## Why is it interesting?

In the paper the authors make a domain simplification along the lines of "the majority of content on the web is English, therefore we can focus on that".
This obviously poses a harsh limitation on the use of this algorithm in the real world and I wanted to know if there was a way around it.

The core idea of this algorithm is essentially that we can project our document's language into a smaller language domain. This can projection could even
be pre-computed if we know the input domain. But, as our input domain grows, so must the output domain in other to be able to represent the full document.

So, what can we do? Well the simplest idea, and the one I've implemented, is a dynamic encoder. It first scanns the document to find out how many unique
UTF-8 characters are present. It then figures out a `code_size` – essentially how wide each number needs to be – then assigns encodings on the fly. For example

```xml
<doc>
  <title>abc<title/>
  <p>defghijklmnopqrstuvwxyz</p>
</doc>
```

Scanning this document, we find that there are `26` unique letters. For this we require `2` wide codes. The mapping would be

```txt
'a' -> '01'
'b' -> '02'
'c' -> '03'
'd' -> '04'
'e' -> '05'
'f' -> '06'
'g' -> '07'
'h' -> '08'
'i' -> '09'
'j' -> '10'
'k' -> '11'
'l' -> '12'
'm' -> '13'
'n' -> '14'
'o' -> '15'
'p' -> '17'
'q' -> '18'
'r' -> '19'
's' -> '20'
't' -> '21'
'u' -> '22'
'v' -> '23'
'y' -> '24'
'x' -> '25'
'z' -> '26'
```

Tags are also represented by numerical strings. These do not require a fixed size, however. For this document it would be

```txt
'<doc>'   -> '0'
'<title>' -> '1'
'<p>'     -> '2'
```

Attributes are handled the same as elements. For the actual encoding we also prefix elements with `T` and attributes with `A`. We also let `0` denote element closure.
Notice also that we'd like to assign common (or long in terms of characters) elements to low numbers. This is also a core part as to why this works.

The encoding looks like this

```txt
T0 T1 000102 0 T2 030405060708091011121314151617181920212223242526 0 0
```

Since our alphabet now only consists of $c \in \{ T, A, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 \}$ we can apply some clever bit packing. Simply assign a 4-bit number to each token of the languge,
and pack two adjacent nibbles into one byte. This step halvs the encoding size. After this we simply let backend compressor algorithm go to town (deflate in my case).

Now, how to we decompress this? Well since my codings are dynamic they need to be stored somewhere. The simplest idea I could think of was to store some header information
just before the final compression step. This can be parsed and used to re-construct the encoder structs during decompression.

## Results

It seem rather useful. But I have not yet had time to test the limits.

It's quite likely efficiency suffers for documents with a wide range of unique characters. My guess is that `3` wide codes would still be fairly useful, but `4` or `5` wide codes would
greatly make compression ratio suffer. However, in practise, I still think it may be useful. You could simply scan a document then make an educated guess as to whether to apply the algorithm or not.











