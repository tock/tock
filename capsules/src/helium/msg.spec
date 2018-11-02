(name "msg")
(version "0.1.0")
(fingerprint 2e1a972830226cbfd75d3bc2531921a9e389298d)
(size 1 200)
(depth 5)
(typelength 1)
(lengthtag t1)
(type
  payload
  vector
  (fingerprint a5b7bdaa64e1985021517b2c24e4232a621998fe)
  (size 1 181)
  (depth 2)
  u8
  180
  t1)
(type
  addr
  array
  (fingerprint ea0dda38212999bb35507220d4bcc0d8aa19efef)
  (size 10 10)
  (depth 2)
  u8
  10)
(type
  ping
  record
  (fingerprint df0b923e395dc033dbd169935fb63a5a9ef92d65)
  (size 18 198)
  (depth 3)
  (fields
    (field id 1 u16)
    (field address 2 addr)
    (field seq 3 u8)
    (field len 4 u32)
    (field data 5 payload)))
(type
  pong
  record
  (fingerprint 70fc746f4d64f4e8d48fb2a9a5fe84cf516b9233)
  (size 16 16)
  (depth 3)
  (fields (field id 1 u8) (field address 2 addr) (field seq 3 u8) (field len 4 u32)))
(type
  pingpong
  union
  (fingerprint 53ad70e5d7563c1758190871cbc422481adfb3b6)
  (size 17 199)
  (depth 4)
  t1
  (fields (field ping 1 ping) (field pong 2 pong)))
(type
  frame
  union
  (fingerprint 4b5f291b6d67e5df2a34f17c5a64ae4647947b86)
  (size 18 200)
  (depth 5)
  t1
  (fields (field pingpong 1 pingpong)))
