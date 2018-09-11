;; -*- cauterize -*-
(name "msg")
(version "0.1.0")

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
    (field ping u32)
    (field pong pong)))

(type pong record
  (fields
    ;; Identity of ponger.
    (field id    u32)
    ;; Sequence sent in `ping`.
    (field seq   u32)))
