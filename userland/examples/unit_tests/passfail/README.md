# Pass/fail unit test

This simple unit test is a designed to validate that the unit test
infrastructure itself works correctly. It provides six tests, which alternate
between passing and failing.


If you load this app along with the `unit_test_supervisor` in `examples/services/`, 
you should see the following console output:

```
1.0: [✓]
1.1: [✓]
1.2: [✓]
1.3: [FAILED]
1.4: [FAILED]
1.5: [✓]
Summary 1: [4/6] Passed, [2/6] Failed, [0/6] Incomplete
```

The tests should be reported in exactly the order given above.
