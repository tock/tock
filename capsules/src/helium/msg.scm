;; -*- cauterize -*-
(name "msg")
(version "0.1.0")

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Field types                                                            ;;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Represents field type which can be used to represent field in frames or params
;; in radio configuration

(type addr array u8 10)
(type payload vector u8 180)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Messages                                                               ;;
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Represents the top-level frame for all message types.
;; In other words, all frames sent/received over the radio
;; will contain one of these variants.
(type frame union
  (fields
    (field pingpong pingpong)))


(type pingpong union
  (fields
    (field ping ping)
    (field pong pong)))

(type ping record
      (fields
        ;; Identity
        (field id   u16)
        ;; Address of team endpoint
        (field address addr)
        ;; Sequence of frame
        (field seq  u8)
        ;; Length of payload
        (field len  u32)
        ;; Payload (can be <= 200 bytes)
        (field data payload)))

(type pong record
  (fields
    ;; Identity of ponger.
    (field id    u8)
    ;; Address of ponger message.
    (field address addr)
    ;; Sequence sent in `ping`.
    (field seq   u8)
    ;; Length of payload
    (field len   u32)))
