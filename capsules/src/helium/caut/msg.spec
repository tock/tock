(name "msg")
(version "0.1.0")
(fingerprint 21d1f3b49cdb0cc41d8cf0b58706bf3b5c5cbe96)
(size 1 10)
(depth 4)
(typelength 1)
(lengthtag t1)
(type 
  pong
  record
  (fingerprint a29e38b79510792abd27a48f284513cd644feaf9)
  (size 8 8)
  (depth 2)
  (fields (field id 1 u32) (field seq 2 u32)))
(type 
  pingpong
  union
  (fingerprint 77966d1390060d7fb5bac850a94dba3229c44fc1)
  (size 5 9)
  (depth 3)
  t1
  (fields (field ping 1 u32) (field pong 2 pong)))
(type 
  frame
  union
  (fingerprint db89e652aa95656d4a7952a5e117ed40e2591555)
  (size 6 10)
  (depth 4)
  t1
  (fields (field pingpong 1 pingpong)))
