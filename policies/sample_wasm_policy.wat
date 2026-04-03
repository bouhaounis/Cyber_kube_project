(module
  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 1024))

  (func (export "alloc") (param $len i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $len)))
    (local.get $ptr))

  (func (export "evaluate") (param $ptr i32) (param $len i32) (result i32)
    ;; Demo policy: treat any non-empty input as a match.
    (if (result i32)
      (i32.gt_s (local.get $len) (i32.const 0))
      (then (i32.const 1))
      (else (i32.const 0)))))
